use serde::Serialize;
use tracing::error;

use crate::search::search_executor::parser::ParsedQuery;
use crate::search::search_executor::query::MSHit;

use super::{Highlighting, Limits};

mod formatter;
mod lexer;
mod merger;
mod parser;
mod query;

#[derive(Serialize, Debug, Clone)]
pub struct ResultsSection {
    facet: String,
    entries: Vec<ResultEntry>,
    n_visible: usize,
    #[serde(rename = "estimatedTotalHits")]
    estimated_total_hits: usize,
}

#[derive(Serialize, Default, Debug, Clone)]
struct ResultEntry {
    #[serde(skip)]
    hit: MSHit,
    id: String,
    r#type: String,
    name: String,
    subtext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtext_bold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parsed_id: Option<String>,
}

pub async fn do_geoentry_search(
    q: String,
    highlighting: Highlighting,
    limits: Limits,
) -> Vec<ResultsSection> {
    let parsed_input = ParsedQuery::from(q.as_str());

    match query::GeoEntryQuery::from((&parsed_input, &limits, &highlighting))
        .execute()
        .await
    {
        Ok(response) => {
            let (section_buildings, mut section_rooms) = merger::merge_search_results(
                &limits,
                response.results.first().unwrap(),
                response.results.get(1).unwrap(),
                response.results.get(2).unwrap(),
            );
            let visitor = formatter::RoomVisitor::from((parsed_input, highlighting));
            section_rooms
                .entries
                .iter_mut()
                .for_each(|r| visitor.visit(r));

            match section_buildings.n_visible {
                0 => vec![section_rooms, section_buildings],
                _ => vec![section_buildings, section_rooms],
            }
        }
        Err(e) => {
            // error should be serde_json::error
            error!("Error searching for results: {e:?}");
            vec![]
        }
    }
}
