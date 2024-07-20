use serde::Serialize;
use tracing::error;

use crate::limited::vec::LimitedVec;
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
#[tracing::instrument]
pub async fn do_geoentry_search(
    q: String,
    highlighting: Highlighting,
    limits: Limits,
) -> LimitedVec<ResultsSection> {
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
                0 => LimitedVec(vec![section_rooms, section_buildings]),
                _ => LimitedVec(vec![section_buildings, section_rooms]),
            }
        }
        Err(e) => {
            // error should be serde_json::error
            error!("Error searching for results: {e:?}");
            LimitedVec(vec![])
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(serde::Deserialize)]
    struct TestQuery {
        target: String,
        query: String,
        among: Option<usize>,
        comment: Option<String>,
    }

    impl TestQuery {
        fn load_good() -> Vec<Self> {
            serde_yaml::from_str(include_str!("test-queries.good.yaml")).unwrap()
        }
        fn load_bad() -> Vec<Self> {
            serde_yaml::from_str(include_str!("test-queries.bad.yaml")).unwrap()
        }
        fn actual_matches_among(&self, actual: &[ResultsSection]) -> bool {
            let among = self.among.unwrap_or(1);
            let mut acceptable_range = actual.iter().flat_map(|r| r.entries.clone()).take(among);
            acceptable_range.any(|r| r.id == self.target)
        }
    }
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_good_queries() {
        let highlighting = Highlighting::default();
        let limits = Limits::default();
        for query in TestQuery::load_good() {
            let info = format!(
                "{query} should get {target}",
                query = query.query,
                target = query.target
            );
            let actual = do_geoentry_search(query.query.clone(), highlighting.clone(), limits)
                .await
                .0;
            assert!(query.actual_matches_among(&actual), "{query} should get {target}. Since it can't, please move it to .bad list, actual={actual:?}", query=query.query, target=query.target);

            insta::with_settings!({
                info => &info,
                description => query.comment.unwrap_or_default(),
            }, {
                        insta::assert_yaml_snapshot!(actual);
            });
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_bad_queries() {
        let highlighting = Highlighting::default();
        let limits = Limits::default();
        for query in TestQuery::load_bad() {
            let info = format!(
                "{query} should get {target}",
                query = query.query,
                target = query.target
            );
            let actual = do_geoentry_search(query.query.clone(), highlighting.clone(), limits)
                .await
                .0;
            assert!(query.actual_matches_among(&actual), "{query} should not be able to get {target}. Since it can't, please move it to .good list, actual={actual:?}", query=query.query, target=query.target);

            insta::with_settings!({
                info => &info,
                description => query.comment.unwrap_or_default(),
            }, {
                insta::assert_yaml_snapshot!(actual);
            });
        }
    }
}
