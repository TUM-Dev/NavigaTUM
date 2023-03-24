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
    target_pos: int
    hits: list[dict[str, Any]]
    num_results: int


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
        if 0 <= self.full_search.target_pos <= 4:
            # "among" may specify where the target can also be without affecting grading
            if self.full_search.target_pos == 0 or (self.full_search.target_pos < self.query.among):
                return 1.0
            preceding_hits = self.full_search.hits[: self.full_search.target_pos]
            preceding_types = {hit["type"] for hit in preceding_hits}
            if self.full_search.hits[self.full_search.target_pos]["type"] in preceding_types:
                return 3.0
            return 2.0
        if 0 < self.full_search.target_pos <= 20:
            return 4.0
        if self.full_search.num_results == 0:
            return 4.7
        return 5.0


class EvaluatableQuery(Query):
    def _do_search(self, search_endpoint: str, length: int) -> SearchResult:
        url = search_endpoint + "?" + urllib.parse.urlencode({"q": self.query[:length]})
        req = requests.get(url).json()

        hits = list(itertools.chain(*[s["entries"] for s in req["sections"]]))
        try:
            target_pos = [s["id"] == self.target for s in hits].index(True)
        except ValueError:
            target_pos = -1

        return SearchResult(
            target_pos=target_pos,
            hits=hits,
            num_results=sum(s["estimatedTotalHits"] for s in req["sections"]),
        )

    def evaluate(self, search_endpoint: str) -> Evaluation:
        """
        Evaluate how well the search engine behaves for a specific query.

        :search_endpoint: The endpoint to query
        """
        full_search: SearchResult = self._do_search(search_endpoint, len(self.query))

        # Apart from the target search we iteratively type the query and assess how much of
        # it is required to reach the position of the final search. Additionally, we can check
        # whether the position is better at any point.
        best_pos: None | int = None
        len_to_best_pos: None | int = None
        len_to_reach_top_5: None | int = None
        len_to_reach_final: None | int = None
        for length in range(1, len(self.query) + 1):
            partial_search: SearchResult = self._do_search(search_endpoint, length)

            if 0 <= partial_search.target_pos:
                if len_to_reach_top_5 is None and 0 <= partial_search.target_pos <= 4:
                    len_to_reach_top_5 = length
                if len_to_reach_final is None and partial_search.target_pos == full_search.target_pos:
                    len_to_reach_final = length

                if best_pos is None or partial_search.target_pos < best_pos:
                    best_pos = partial_search.target_pos
                    len_to_best_pos = length

        return Evaluation(
            query=self,
            best_pos=best_pos,
            len_to_reach_top_5=len_to_reach_top_5,
            len_to_reach_final=len_to_reach_final,
            len_to_best_pos=len_to_best_pos,
            full_search=full_search,
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
        return thread_map(lambda query: query.evaluate(self.search_endpoint), self.queries,
                          desc=f"Querying {self.search_endpoint}")


if __name__ == "__main__":
    current = Evaluations(search_endpoint="https://nav.tum.de/api/search")
    comparison = Evaluations(search_endpoint="https://nav.tum.de/api/search")

    printer.print_specific_queries_result(current.results, comparison.results)
