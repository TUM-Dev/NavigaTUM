import search_quality_test
from termcolor import colored


def print_specific_queries_result(
    current: list[search_quality_test.Evaluation],
    comparison: list[search_quality_test.Evaluation],
):
    """Print the results of the specific queries"""
    comparison_dict = {comp.query: comp for comp in comparison}
    for search in current:
        comp = comparison_dict[search.query]
        s_pos_indicator = _gen_pos_indicator(search)

        # Grade
        s_grade = {
            "1.0": colored("1.0", "green", attrs=["bold"]),
            "2.0": colored("2.0", "green", attrs=[]),
            "3.0": colored("3.0", "yellow", attrs=["bold"]),
            "4.0": colored("4.0", "yellow", attrs=[]),
            "4.7": colored("4.7", "red", attrs=[]),
            "5.0": colored("5.0", "red", attrs=["bold"]),
        }[str(round(search.grade, 1))]

        # Grade cmp
        s_cmp = _generate_grade_cmp(search, comp)

        s_query = _gen_colored_query(search)
        s_stats = _gen_colored_stats(search)

        print(f"{s_pos_indicator} {s_grade}{s_cmp} {s_query} {s_stats}")

    num_searches = sum(len(s.query.query) + 1 for s in current)
    print(f"Performed {num_searches} searches")


def _gen_colored_query(search: search_quality_test.Evaluation):
    """
    Generates the colored Query
    - Green indicates when a better position is reached
    - White (not formatted) indicates minimum to reach top 5
    - Underline indicates minimum to reach final position
    """
    green_end_pos = (
        search.len_to_best_pos
        if (
            search.best_pos is not None
            and search.len_to_best_pos is not None
            and search.full_search.target_pos is not None
            and (not search.full_search.was_successful or search.best_pos < search.full_search.target_pos)
        )
        else 0
    )
    white_end_pos = search.len_to_reach_top_5 if search.len_to_reach_top_5 is not None else 0
    underline_end_pos = search.len_to_reach_final if search.len_to_reach_final is not None else 0

    s_query = ""
    for i, query_char in enumerate(search.query.query):
        # This is not the best way of formatting, but sufficient here
        if i >= green_end_pos and i >= white_end_pos:
            s_query += colored(
                str(query_char),
                color="white",  # this is gray
                attrs=(["underline"] if i < underline_end_pos else []),
            )
        elif green_end_pos < white_end_pos:
            s_query += colored(
                str(query_char),
                color="green" if i < green_end_pos else None,
                attrs=(["underline"] if i < underline_end_pos else []),
            )
        else:
            s_query += colored(
                str(query_char),
                color=None if i < white_end_pos else "green",
                attrs=(["underline"] if i < underline_end_pos else []),
            )
    s_query += " " * max(0, 50 - len(search.query.query))
    return s_query


def _generate_grade_cmp(search: search_quality_test.Evaluation, comp: search_quality_test.Evaluation):
    grade_diff = abs(round(search.grade - comp.grade, 1))
    if comp.grade > search.grade:
        return colored(f" +{grade_diff}", "red", attrs=["bold"])
    if comp.grade < search.grade:
        return colored(f" -{grade_diff}", "green", attrs=["bold"])
    return "     "


def _gen_colored_stats(search: search_quality_test.Evaluation):
    """Generate the colored statistics"""
    num_results = search.full_search.num_results
    return f"{num_results:>4}" + colored(" hits, target: '", "white") + search.query.target + colored("')", "white")


def _gen_pos_indicator(search: search_quality_test.Evaluation):
    """The position indicator shows rougly how the results looked like and where the target entry was located"""
    if search.full_search.was_top5:
        target_pos = search.full_search.target_pos or 0
        return (
            colored("[", "white")
            + " " * target_pos
            + colored("*", "cyan", attrs=["bold"])
            + " " * (min(search.full_search.num_results, 5) - target_pos - 1)
            + colored("]", "white")
            + colored("-" * (5 - min(search.full_search.num_results, 5)), "white")
            + " "
        )
    if search.full_search.was_top20:
        return colored("[     ]", "white") + colored(">", "yellow") + " "
    if search.full_search.num_results > 0:
        return (
            colored("[", "white")
            + colored("x" * min(search.full_search.num_results, 5), "red")
            + colored("]", "white")
            + colored("-" * (5 - min(search.full_search.num_results, 5)), "white")
            + " "
        )
    return colored("[]-----", "red") + " "
