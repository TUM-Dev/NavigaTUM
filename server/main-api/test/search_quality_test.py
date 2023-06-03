import dataclasses
import itertools
import urllib.parse
from pathlib import Path
from typing import Any

import printer
import requests
import yaml
from tqdm.contrib.concurrent import thread_map


@dataclasses.dataclass
class Query:  # split into Query and EvaluatableQuery for import reasons
    target: str
    query: str
    among: int = 1

    def __hash__(self):
        return hash((self.target, self.query, self.among))


@dataclasses.dataclass
class SearchResult:
    queried_length: int
    target_pos: int | None
    hits: list[dict[str, Any]]
    num_results: int

    @property
    def was_successful(self):
        """Whether the search was successful (i.e. there were results)"""
        return self.target_pos is not None

    @property
    def was_top5(self):
        """Whether the target was among the first 5 results"""
        return self.target_pos is not None and 0 <= self.target_pos < 5

    @property
    def was_top20(self):
        """Whether the target was among the first 20 results"""
        return self.target_pos is not None and 0 <= self.target_pos < 20


@dataclasses.dataclass
class Evaluation:
    query: Query
    best_pos: int | None
    len_to_reach_top_5: int | None
    len_to_reach_final: int | None
    len_to_best_pos: int | None
    full_search: SearchResult

    @property
    def grade(self):
        """
        Calculate the grade of a search.

        Grading Rubric:
          1.0 = excellent (e.g. first result)
          2.0 = good (e.g. the first result of its type)
          3.0 = okay (e.g. among the first 5 results, some other results of its type before it)
          4.0 = passed (e.g. not in the first 5 results in autocomplete, but within the first 20 results)
          4.7 = failed (but at least not misleading, e.g. when there are no results at all)
          5.0 = very bad (e.g. not in the results, misleading results)
        """
        if self.full_search.was_top5:
            # "among" may specify where the target can also be without affecting grading
            if self.full_search.target_pos < self.query.among:
                return 1.0
            preceding_hits = self.full_search.hits[: self.full_search.target_pos]
            preceding_types = {hit["type"] for hit in preceding_hits}
            if self.full_search.hits[self.full_search.target_pos]["type"] not in preceding_types:
                return 2.0
            return 3.0
        if self.full_search.was_top20:
            return 4.0
        if self.full_search.num_results == 0:
            return 4.7
        return 5.0


class EvaluatableQuery(Query):
    def do_search(self, search_endpoint: str, length: int) -> SearchResult:
        """
        Perform a search for a specific query
        """
        url = search_endpoint + "?" + urllib.parse.urlencode({"q": self.query[:length]})
        req = requests.get(url, timeout=10).json()

        hits = list(itertools.chain(*[s["entries"] for s in req["sections"]]))
        try:
            target_pos = [s["id"] == self.target for s in hits].index(True)
        except ValueError:
            target_pos = None

        return SearchResult(
            queried_length=length,
            target_pos=target_pos,
            hits=hits,
            num_results=sum(s["estimatedTotalHits"] for s in req["sections"]),
        )

    def evaluate(self, search_endpoint: str) -> Evaluation:
        """
        Evaluate how well the search engine behaves for a specific query.

        :search_endpoint: The endpoint to query
        """
        query_lengths = range(1, len(self.query) + 1)
        searches = [self.do_search(search_endpoint, length) for length in query_lengths]
        final_search = searches[-1]
        successfull_searches = [search for search in searches if search.was_successful]
        # among all successfull searches we look at the first one that reaches the best position
        best_search = min(
            successfull_searches,
            key=lambda ls: (ls.target_pos, ls.queried_length),
            default=None,
        )

        # among all successfull searches we look at the first one that reaches the top 5
        searches_who_reached_top_5 = [search for search in successfull_searches if search.was_top5]
        len_to_reach_top_5 = searches_who_reached_top_5[0].queried_length if searches_who_reached_top_5 else None

        # among all successfull searches we look at the first one that reaches the final position
        len_to_reach_final = min(
            (search.queried_length for search in successfull_searches if search.target_pos == final_search.target_pos),
            default=None,
        )

        return Evaluation(
            query=self,
            best_pos=best_search.target_pos if best_search else None,
            len_to_best_pos=best_search.queried_length if best_search else None,
            len_to_reach_top_5=len_to_reach_top_5,
            len_to_reach_final=len_to_reach_final,
            full_search=final_search,
        )


@dataclasses.dataclass
class Evaluations:
    search_endpoint: str

    @property
    def queries(self) -> list[Query]:
        """Load the queries from the test-queries.yaml file"""
        with open(Path(__file__).parent / "test-queries.yaml", encoding="utf-8") as file:
            return [EvaluatableQuery(**query) for query in yaml.safe_load(file.read())]

    @property
    def results(self) -> list[Evaluation]:
        """Test the specific queries, and return the results"""
        return thread_map(
            lambda query: query.evaluate(self.search_endpoint),
            self.queries,
            desc=f"Querying {self.search_endpoint}",
        )


if __name__ == "__main__":
    current = Evaluations(search_endpoint="https://nav.tum.de/api/search")
    comparison = Evaluations(search_endpoint="https://nav.tum.de/api/search")

    printer.print_specific_queries_result(current.results, comparison.results)
