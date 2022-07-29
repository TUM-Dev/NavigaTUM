import itertools
import json
import os
import urllib.parse

import requests
import yaml
from progress.bar import Bar  # type: ignore
from termcolor import colored, cprint

SEARCH_ENDPOINT = "http://localhost:8080/api/search"


def _do_search(qst: str, query: dict):
    url = SEARCH_ENDPOINT + "?" + urllib.parse.urlencode({"q": qst})
    req = requests.get(url).json()

    search = {
        "query": query["query"],
        "target": query["target"],
        "hits": list(itertools.chain(*[s["entries"] for s in req["sections"]])),
        "num_results": sum([s["estimatedTotalHits"] for s in req["sections"]]),
        "time_ms": req["time_ms"],
    }

    try:
        search["target_pos"] = [s["id"] == query["target"] for s in search["hits"]].index(True)
    except ValueError:
        search["target_pos"] = -1

    return search


def test_specific_queries(queries):
    """Test the specific queries, and return the results"""
    searches = []

    for query in Bar("Querying", suffix="%(index)d / %(max)d").iter(queries):
        search = _do_search(query["query"], query)

        # Apart from the target search we iteratively type the query and assess how much of
        # it is required to reach the position of the final search. Additionally, we can check
        # whether the position is better at any point.
        query_times = []
        best_pos = None
        len_to_best_pos = None
        len_to_reach_top_5 = None
        len_to_reach_final = None
        for length in range(1, len(query["query"]) + 1):
            partial_search = _do_search(query["query"][:length], query)

            if 0 <= partial_search["target_pos"]:
                if len_to_reach_top_5 is None and 0 <= partial_search["target_pos"] <= 4:
                    len_to_reach_top_5 = length
                if len_to_reach_final is None and partial_search["target_pos"] == search["target_pos"]:
                    len_to_reach_final = length

                if best_pos is None or partial_search["target_pos"] < best_pos:
                    best_pos = partial_search["target_pos"]
                    len_to_best_pos = length

            query_times.append(partial_search["time_ms"])

        search["best_pos"] = best_pos
        search["len_to_reach_top_5"] = len_to_reach_top_5
        search["len_to_reach_final"] = len_to_reach_final
        search["len_to_best_pos"] = len_to_best_pos

        search["partial_time_avg"] = sum(query_times) / len(query_times)
        search["partial_time_max"] = max(query_times)

        search["grade"] = _calculate_grade(query, search)

        searches.append(search)

    return searches


def _calculate_grade(query, search):
    """
    Calculate the grade of a search.

    Grading:
        1.0 = excellent (e.g. first result)
        2.0 = good (e.g. the first result of its type)
        3.0 = okay (e.g. among the first 5 results, some other results of its type before it)
        4.0 = passed (e.g. not in the first 5 results in autocomplete, but within the first 20 results)
        4.7 = failed (but at least not misleading, e.g. when there are no results at all)
        5.0 = very bad (e.g. not in the results, misleading results)
    """
    if 0 <= search["target_pos"] <= 4:
        # "among" may specify where the target can also be without affecting grading
        if search["target_pos"] == 0 or ("among" in query and search["target_pos"] < query["among"]):
            return 1.0
        preceding_hits = search["hits"][: search["target_pos"]]
        preceding_types = {hit["type"] for hit in preceding_hits}
        if search["hits"][search["target_pos"]]["type"] in preceding_types:
            return 3.0
        return 2.0
    if 0 < search["target_pos"] <= 20:
        return 4.0
    if search["num_results"] == 0:
        return 4.7
    return 5.0


def _print_specific_queries_result(searches, cmp=None):
    """Print the results of the specific queries"""
    for search in searches:
        s_pos_indicator = _gen_pos_indicator(search)

        # Grade
        s_grade = {
            "1.0": colored("1.0", "green", attrs=["bold"]),
            "2.0": colored("2.0", "green", attrs=[]),
            "3.0": colored("3.0", "yellow", attrs=["bold"]),
            "4.0": colored("4.0", "yellow", attrs=[]),
            "4.7": colored("4.7", "red", attrs=[]),
            "5.0": colored("5.0", "red", attrs=["bold"]),
        }[str(round(search["grade"], 1))]

        # Grade cmp
        s_cmp = _generate_grade_cmp(cmp, search)

        s_query = _gen_colored_query(search)
        s_stats = _gen_colored_stats(search)

        print(f"{s_pos_indicator} {s_grade}{s_cmp} {s_query} {s_stats}")

    num_searches = sum(len(s["query"]) + 1 for s in searches)
    total_search_times = sum(s["partial_time_avg"] * len(s["query"]) for s in searches)
    avg_search_times = total_search_times / sum(len(s["query"]) for s in searches)
    print(f"Performed {num_searches} searches, {round(avg_search_times, 1)}ms (partial) average")


def _gen_colored_query(search):
    """
    Generates the colored Query
    - Green indicates when a better position is reached
    - White (not formatted) indicates minimum to reach top 5
    - Underline indicates minimum to reach final position
    """
    green_end_pos = (
        search["len_to_best_pos"]
        if (
            search["best_pos"] is not None and (search["target_pos"] == -1 or search["best_pos"] < search["target_pos"])
        )
        else 0
    )
    white_end_pos = search["len_to_reach_top_5"] if search["len_to_reach_top_5"] is not None else 0
    underline_end_pos = search["len_to_reach_final"] if search["len_to_reach_final"] is not None else 0

    s_query = ""
    for i, query in enumerate(search["query"]):
        # This is not the best way of formatting, but sufficient here
        if i >= green_end_pos and i >= white_end_pos:
            s_query += colored(
                str(query),
                color="white",  # this is gray
                attrs=(["underline"] if i < underline_end_pos else []),
            )
        elif green_end_pos < white_end_pos:
            s_query += colored(
                str(query),
                color="green" if i < green_end_pos else None,
                attrs=(["underline"] if i < underline_end_pos else []),
            )
        else:
            s_query += colored(
                str(query),
                color=None if i < white_end_pos else "green",
                attrs=(["underline"] if i < underline_end_pos else []),
            )
    s_query += " " * max(0, 50 - len(search["query"]))
    return s_query


def _generate_grade_cmp(cmp, search):
    if cmp is None:
        return ""
    cmp_search = None
    for comparison in cmp:
        if comparison["query"].lower() == search["query"].lower():
            cmp_search = comparison
            break

    if cmp_search is None:
        return colored(" ----", "white")
    if cmp_search["grade"] < search["grade"]:
        grade = round(search["grade"] - cmp_search["grade"], 1)
        return colored(f" +{grade}", "red", attrs=["bold"])
    if cmp_search["grade"] > search["grade"]:
        grade = round(cmp_search["grade"] - search["grade"], 1)
        return colored(f" -{grade}", "green", attrs=["bold"])
    return "     "


def _gen_colored_stats(search):
    """Generate the colored statistics"""
    time_ms = search["time_ms"]
    partial_time_avg = round(search["partial_time_avg"])
    partial_time_max = search["partial_time_max"]
    num_results = search["num_results"]
    # Stats
    return (
        colored("(", "white")
        + f"{time_ms:>2}"
        + colored("ms [partial avg ", "white")
        + f"{partial_time_avg:>2}"
        + colored(", max ", "white")
        + f"{partial_time_max:>2}"
        + colored("], ", "white")
        + f"{num_results:>4}"
        + colored(" hits, target: '", "white")
        + search["target"]
        + colored("')", "white")
    )


def _gen_pos_indicator(search):
    """The position indicator shows rougly how the results looked like and where the target entry was located"""
    if 0 <= search["target_pos"] <= 4:
        return (
            colored("[", "white")
            + " " * search["target_pos"]
            + colored("*", "cyan", attrs=["bold"])
            + " " * (min(search["num_results"], 5) - search["target_pos"] - 1)
            + colored("]", "white")
            + colored("-" * (5 - min(search["num_results"], 5)), "white")
            + " "
        )
    if 5 <= search["target_pos"] <= 20:
        return colored("[     ]", "white") + colored(">", "yellow") + " "
    if search["num_results"] > 0:
        return (
            colored("[", "white")
            + colored("x" * min(search["num_results"], 5), "red")
            + colored("]", "white")
            + colored("-" * (5 - min(search["num_results"], 5)), "white")
            + " "
        )
    return colored("[]-----", "red") + " "


def main():
    """Main function"""
    with open(os.path.join(os.path.dirname(__file__), "test-queries.yaml"), encoding="utf-8") as file:
        test_queries = yaml.safe_load(file.read())
    cprint("=== Specific queries ===", attrs=["bold"])
    # with open(os.path.join(os.path.dirname(__file__), "cmp-210811.json"), encoding="utf-8") as file:
    #    cmp = json.load(file)
    cmp = None
    searches = test_specific_queries(test_queries["entry_queries"])
    _print_specific_queries_result(searches, cmp)
    with open(os.path.join(os.path.dirname(__file__), "cmp-210811.json"), "w", encoding="utf-8") as file:
        json.dump(searches, file)


if __name__ == "__main__":
    main()
