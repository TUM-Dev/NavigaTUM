use serde::Deserialize;
use std::usize;

#[derive(Debug, Clone)]
struct SearchResult {
    queried_length: usize,
    target_pos: Option<usize>,
    estimated_total_hits_sum: i32,
}

impl SearchResult {
    fn was_successful(&self) -> bool {
        self.target_pos.is_some()
    }

    fn was_top5(&self) -> bool {
        match self.target_pos {
            Some(pos) => pos < 5,
            None => false,
        }
    }

    fn was_top20(&self) -> bool {
        match self.target_pos {
            Some(pos) => pos < 20,
            None => false,
        }
    }
}

#[derive(Debug)]
struct Evaluation {
    query: Query,
    best_pos: Option<usize>,
    len_to_reach_top_5: Option<usize>,
    len_to_reach_final: Option<usize>,
    len_to_best_pos: Option<usize>,
    full_search: SearchResult,
}

#[derive(Debug, Clone, Deserialize)]
struct Query {
    target: String,
    query: String,
    among: Option<usize>,
}

impl Query {
    async fn do_search(&self, url: &str, queried_length: usize) -> SearchResult {
        let req = reqwest::get(url)
            .await
            .expect("Failed to perform the search request")
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse JSON response");

        let hits: Vec<serde_json::Value> = req["sections"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .flat_map(|s| s["entries"].as_array().unwrap_or(&vec![]).to_vec())
            .collect();

        let target_pos = hits
            .iter()
            .position(|s| s["id"].as_str() == Some(&self.target));

        SearchResult {
            queried_length,
            target_pos,
            estimated_total_hits_sum: req["sections"]
                .as_array()
                .map(|sections| {
                    sections
                        .iter()
                        .map(|s| s["estimatedTotalHits"].as_i64().unwrap_or(0))
                        .sum()
                })
                .unwrap_or(0) as i32,
        }
    }

    async fn evaluate(self, search_endpoint: &str) -> Evaluation {
        let mut searches: Vec<SearchResult> = vec![];
        for query_length in 1..=self.query.len() {
            let url = format!("{search_endpoint}/?q={}", &self.query[..query_length]);
            searches.push(self.do_search(&url, query_length).await);
        }

        let final_search = searches.last().unwrap().clone();
        let successfull_searches: Vec<SearchResult> = searches
            .into_iter()
            .filter(|search| search.was_successful())
            .collect();

        let best_search = successfull_searches
            .iter()
            .min_by_key(|&search| {
                (
                    search.target_pos.unwrap_or(usize::MAX),
                    search.queried_length,
                )
            })
            .cloned();

        let searches_who_reached_top_5: Vec<SearchResult> = successfull_searches
            .clone()
            .into_iter()
            .filter(|search| search.was_top5())
            .collect();

        let len_to_reach_top_5 = searches_who_reached_top_5
            .first()
            .map(|search| search.queried_length);

        let len_to_reach_final = successfull_searches
            .iter()
            .filter(|&search| search.target_pos == final_search.target_pos)
            .map(|search| search.queried_length)
            .min();

        Evaluation {
            query: self,
            best_pos: best_search
                .clone()
                .map(|search| search.target_pos.unwrap_or(usize::MAX)),
            len_to_reach_top_5,
            len_to_reach_final,
            len_to_best_pos: best_search.map(|search| search.queried_length),
            full_search: final_search.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queries() -> Vec<Query> {
        let test_queries = include_str!("test-queries.yaml");
        serde_yaml::from_str::<Vec<Query>>(&test_queries)
            .expect("Failed to parse test-queries.yaml")
    }
    #[tokio::test]
    async fn evaluate_queries() {
        let search_endpoint = "https://nav.tum.de/api/search";
        for query in queries() {
            let evaluation = query.evaluate(search_endpoint).await;
            println!("{evaluation:?}");
        }
    }
}
