use super::SanitisedSearchQueryArgs;
use cached::proc_macro::cached;
use log::error;

mod lexer;
mod parser;
mod postprocess;
mod query;
use crate::search::search_executor::parser::ParsedQuery;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct SearchResultsSection {
    facet: String,
    entries: Vec<ResultEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_visible: Option<usize>,
    #[serde(rename = "estimatedTotalHits")]
    estimated_total_hits: usize,
}

#[derive(Serialize, Debug, Clone)]
struct ResultEntry {
    id: String,
    r#type: String,
    name: String,
    subtext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtext_bold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parsed_id: Option<String>,
}

// size=2500 seems to be about 250Mi
#[cached(size = 2500)]
pub async fn do_geoentry_search(
    q: String,
    highlighting: (String, String),
    args: SanitisedSearchQueryArgs,
) -> Vec<SearchResultsSection> {
    let parsed_input = ParsedQuery::from(q.as_str());

    match query::GeoEntryQuery::from(&parsed_input, &args, &highlighting)
        .execute()
        .await
    {
        Ok(response) => postprocess::merge_search_results(
            &args,
            &parsed_input,
            response.results.get(0).unwrap(),
            response.results.get(1).unwrap(),
            response.results.get(2).unwrap(),
            highlighting,
        ),
        Err(e) => {
            // error should be serde_json::error
            error!("Error searching for results: {e:?}");
            vec![]
        }
    }
}
