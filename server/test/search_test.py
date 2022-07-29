import itertools
import json
import os
import sys
import urllib

import requests
import yaml
from progress.bar import Bar
from termcolor import colored, cprint

SEARCH_ENDPOINT = "http://localhost:8080/api/search"


def test_specific_queries(queries):
    searches = []

    for q in Bar("Querying", suffix="%(index)d / %(max)d").iter(queries):
        target_id = q["target"]

        def do_search(q_str):
            url = SEARCH_ENDPOINT + "?" + urllib.parse.urlencode({"q": q_str})
            r = requests.get(url).json()

            search = {
                "query": q["query"],
                "target": q["target"],
                "hits": list(itertools.chain(*[s["entries"] for s in r["sections"]])),
                "num_results": sum([s["estimatedTotalHits"] for s in r["sections"]]),
                "time_ms": r["time_ms"],
            }

            try:
                search["target_pos"] = list(map(lambda s: s["id"] == target_id, search["hits"])).index(True)
            except ValueError:
                search["target_pos"] = -1

            return search

        search = do_search(q["query"])

        # Apart from the target search we iterativeley type the query and assess how much of
        # it is required to reach the position of the final search. Additionally we can check
        # whether the position is better at any point.
        query_times = []
        best_pos = None
        len_to_best_pos = None
        len_to_reach_top_5 = None
        len_to_reach_final = None
        for l in range(1, len(q["query"]) + 1):
            partial_search = do_search(q["query"][:l])

            if 0 <= partial_search["target_pos"]:
                if len_to_reach_top_5 is None and 0 <= partial_search["target_pos"] <= 4:
                    len_to_reach_top_5 = l
                if len_to_reach_final is None and partial_search["target_pos"] == search["target_pos"]:
                    len_to_reach_final = l

                if best_pos is None or partial_search["target_pos"] < best_pos:
                    best_pos = partial_search["target_pos"]
                    len_to_best_pos = l

            query_times.append(partial_search["time_ms"])

        search["best_pos"] = best_pos
        search["len_to_reach_top_5"] = len_to_reach_top_5
        search["len_to_reach_final"] = len_to_reach_final
        search["len_to_best_pos"] = len_to_best_pos

        search["partial_time_avg"] = sum(query_times) / len(query_times)
        search["partial_time_max"] = max(query_times)

        # Grading:
        # 1.0 = excellent (e.g. first result)
        # 2.0 = good (e.g. the first result of its type)
        # 3.0 = okay (e.g. among the first 5 results, some other results of its type before it)
        # 4.0 = passed (e.g. not in the first 5 results in autocomplete, but within the first 20 results)
        # 4.7 = failed (but at least not misleading, e.g. when there are no results at all)
        # 5.0 = very bad (e.g. not in the results, misleading results)
        if search["target_pos"] == 0:
            search["grade"] = 1.0
        elif 0 < search["target_pos"] <= 4:
            # "among" may specify where the target can also be without affecting grading
            if "among" in q and search["target_pos"] < q["among"]:
                search["grade"] = 1.0
            else:
                preceding_types = set(map(lambda hit: hit["type"], search["hits"][: search["target_pos"]]))
                if search["hits"][search["target_pos"]]["type"] in preceding_types:
                    search["grade"] = 3.0
                else:
                    search["grade"] = 2.0
        elif 0 < search["target_pos"] <= 20:
            search["grade"] = 4.0
        else:
            if search["num_results"] == 0:
                search["grade"] = 4.7
            else:
                search["grade"] = 5.0

        searches.append(search)

    return searches


def _print_specific_queries_result(searches, cmp=None):
    individual_result_strings = []
    for search in searches:
        # The position indicator shows rougly how the results looked like
        # and where the target entry was located
        if 0 <= search["target_pos"] <= 4:
            s_pos_indicator = (
                colored("[", "white")
                + " " * search["target_pos"]
                + colored("*", "cyan", attrs=["bold"])
                + " " * (min(search["num_results"], 5) - search["target_pos"] - 1)
                + colored("]", "white")
                + colored("-" * (5 - min(search["num_results"], 5)), "white")
                + " "
            )
        elif 5 <= search["target_pos"] <= 20:
            s_pos_indicator = colored("[     ]", "white") + colored(">", "yellow") + " "
        elif search["num_results"] > 0:
            s_pos_indicator = (
                colored("[", "white")
                + colored("x" * min(search["num_results"], 5), "red")
                + colored("]", "white")
                + colored("-" * (5 - min(search["num_results"], 5)), "white")
                + " "
            )
        else:
            s_pos_indicator = colored("[]-----", "red") + " "

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
        if cmp is None:
            s_cmp = ""
        else:
            cmp_search = None
            for s in cmp:
                if s["query"].lower() == search["query"].lower():
                    cmp_search = s
                    break

            if cmp_search is None:
                s_cmp = colored(" ----", "white")
            else:
                if cmp_search["grade"] < search["grade"]:
                    s_cmp = colored(
                        " +{}".format(round(search["grade"] - cmp_search["grade"], 1)),
                        "red",
                        attrs=["bold"],
                    )
                elif cmp_search["grade"] > search["grade"]:
                    s_cmp = colored(
                        " -{}".format(round(cmp_search["grade"] - search["grade"], 1)),
                        "green",
                        attrs=["bold"],
                    )
                else:
                    s_cmp = "     "

        # Query
        # Green indicates when a better position is reached
        # White (not formatted) indicates minimum to reach top 5
        # Underline indicates minimum to reach final position
        green_end_pos = (
            search["len_to_best_pos"]
            if (
                search["best_pos"] is not None
                and (search["target_pos"] == -1 or search["best_pos"] < search["target_pos"])
            )
            else 0
        )
        white_end_pos = search["len_to_reach_top_5"] if search["len_to_reach_top_5"] is not None else 0
        underline_end_pos = search["len_to_reach_final"] if search["len_to_reach_final"] is not None else 0

        s_query = ""
        for i, c in enumerate(search["query"]):
            # This is not the best way of formatting, but sufficient here
            if i >= green_end_pos and i >= white_end_pos:
                s_query += colored(
                    str(c),
                    color="white",  # this is gray
                    attrs=(["underline"] if i < underline_end_pos else []),
                )
            elif green_end_pos < white_end_pos:
                s_query += colored(
                    str(c),
                    color="green" if i < green_end_pos else None,
                    attrs=(["underline"] if i < underline_end_pos else []),
                )
            else:
                s_query += colored(
                    str(c),
                    color=None if i < white_end_pos else "green",
                    attrs=(["underline"] if i < underline_end_pos else []),
                )
        s_query += " " * max(0, 50 - len(search["query"]))

        # Stats
        s_stats = (
            colored("(", "white")
            + "{:>2}".format(search["time_ms"])
            + colored("ms [partial avg ", "white")
            + "{:>2}".format(round(search["partial_time_avg"]))
            + colored(", max ", "white")
            + "{:>2}".format(search["partial_time_max"])
            + colored("], ", "white")
            + "{:>4}".format(search["num_results"])
            + colored(" hits, target: '", "white")
            + search["target"]
            + colored("')", "white")
        )

        print(
            "{} {}{} {} {}".format(
                s_pos_indicator,
                s_grade,
                s_cmp,
                s_query,
                s_stats,
            ),
        )

    num_searches = sum(map(lambda s: len(s["query"]) + 1, searches))
    avg_search_times = sum(map(lambda s: s["partial_time_avg"] * len(s["query"]), searches)) / sum(
        map(lambda s: len(s["query"]), searches),
    )
    print("Performed {} searches, {}ms (partial) average".format(num_searches, round(avg_search_times, 1)))


if __name__ == "__main__":
    with open(os.path.join(os.path.dirname(__file__), "test-queries.yaml")) as f:
        test_queries = yaml.safe_load(f.read())

    cprint("=== Specific queries ===", attrs=["bold"])
    # with open(os.path.join(os.path.dirname(__file__), "cmp-210811.json")) as f:
    #    cmp = json.load(f)
    cmp = None

    searches = test_specific_queries(test_queries["entry_queries"])
    _print_specific_queries_result(searches, cmp)

    with open(os.path.join(os.path.dirname(__file__), "cmp-210811.json"), "w") as f:
        json.dump(searches, f)
