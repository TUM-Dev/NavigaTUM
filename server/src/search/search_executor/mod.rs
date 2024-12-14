use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
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

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ResultFacet {
    SitesBuildings,
    Rooms,
    Addresses,
}

#[derive(Serialize, Clone)]
pub struct ResultsSection {
    pub(crate) facet: ResultFacet,
    entries: Vec<ResultEntry>,
    n_visible: usize,
    #[serde(rename = "estimatedTotalHits")]
    estimated_total_hits: usize,
}

impl Debug for ResultsSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_set();
        for i in 0..=3 {
            if let Some(e) = self.entries.get(i) {
                base.entry(e);
            }
        }
        if self.entries.len() > 3 {
            base.entry(&"...");
        }
        base.finish()
    }
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

#[derive(Deserialize, Clone)]
struct NominatimAddressResponse {
    //postcode: Option<String>,
    // country: Option<String>,
    // country_code: Option<String>,
    // ISO3166-2-lvl4: Option<String>,
    state: Option<String>,
    county: Option<String>,
    town: Option<String>,
    suburb: Option<String>,
    village: Option<String>,
    hamlet: Option<String>,
    road: Option<String>,
}

impl NominatimAddressResponse {
    fn serialise(&self) -> String {
        let mut result = Vec::<String>::new();
        if let Some(state) = self.state.clone() {
            result.push(state);
        }
        if let Some(county) = self.county.clone() {
            result.push(county);
        }
        if let Some(town) = self.town.clone() {
            result.push(town);
        }
        if let Some(suburb) = self.suburb.clone() {
            result.push(suburb);
        }
        if let Some(village) = self.village.clone() {
            result.push(village);
        }
        if let Some(hamlet) = self.hamlet.clone() {
            result.push(hamlet);
        }
        if let Some(road) = self.road.clone() {
            result.push(road);
        }
        result.join(", ")
    }
}

#[derive(Deserialize, Clone)]
struct NominatimResponse {
    /// Example: 371651568
    osm_id: i64,
    /// Example: "road",
    #[serde(rename = "addresstype")]
    address_type: String,
    /// Example: "Münchner Straße",
    name: String,
    address: NominatimAddressResponse,
}

#[tracing::instrument]
pub async fn address_search(q: &str) -> LimitedVec<ResultsSection> {
    let url = std::env::var("NOMINATIM_URL")
        .unwrap_or_else(|_| "https://nav.tum.de/nominatim".to_string());
    let url = format!("{url}/search?q={q}&addressdetails=1");
    let Ok(nominatim_results) = reqwest::get(&url).await else {
        error!("cannot get {url}");
        return LimitedVec::from(vec![]);
    };
    let Ok(results) = nominatim_results.json::<Vec<NominatimResponse>>().await else {
        error!("the results from nomnatim is not what we expected {url}");
        return LimitedVec::from(vec![]);
    };
    let num_results = results.len();
    let section = ResultsSection {
        facet: ResultFacet::Addresses,
        entries: results
            .into_iter()
            .map(|r| {
                let subtext = r.address.serialise();
                ResultEntry {
                    hit: Default::default(),
                    id: format!("osm_{}", r.osm_id),
                    r#type: r.address_type,
                    name: r.address.road.unwrap_or(r.name),
                    subtext,
                    subtext_bold: None,
                    parsed_id: None,
                }
            })
            .collect(),
        n_visible: num_results.min(15),
        estimated_total_hits: num_results,
    };
    LimitedVec::from(vec![section])
}

#[tracing::instrument(skip(client))]
pub async fn do_geoentry_search(
    client: &Client,
    q: &str,
    highlighting: Highlighting,
    limits: Limits,
) -> LimitedVec<ResultsSection> {
    let parsed_input = ParsedQuery::from(q);

    let Ok(response) = query::GeoEntryQuery::from((client, &parsed_input, &limits, &highlighting))
        .execute()
        .await
    else {
        // error should be serde_json::error
        error!("Error searching for results");
        return LimitedVec(vec![]);
    };
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::setup::tests::MeiliSearchTestContainer;
    use std::fmt::{Display, Formatter};

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
        async fn search(&self, client: &Client) -> Vec<ResultsSection> {
            do_geoentry_search(
                client,
                &self.query,
                Highlighting::default(),
                Limits::default(),
            )
            .await
            .0
        }
    }
    impl Display for TestQuery {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "'{query}' should get '{target}' in {among} results",
                query = self.query,
                target = self.target,
                among = self.among.unwrap_or(1),
            )?;
            if let Some(comment) = &self.comment {
                write!(f, " # {comment}")?;
            }
            Ok(())
        }
    }
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_good_queries() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();
        for query in TestQuery::load_good() {
            let actual = query.search(&ms.client).await;
            assert!(
                query.actual_matches_among(&actual),
                "{query}\n\
                Since it can't, please move it to .bad list"
            );

            // redacting estimatedTotalHits is because this can change quite easily and without a good reason
            insta::with_settings!({
                info => &format!("{query}"),
                description => query.comment.unwrap_or_default(),
            }, {
                insta::assert_yaml_snapshot!(actual, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
            });
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_bad_queries() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();
        for query in TestQuery::load_bad() {
            let actual = query.search(&ms.client).await;
            assert!(
                !query.actual_matches_among(&actual),
                "{query}\n\
                Since it can, please move it to .good list"
            );

            // redacting estimatedTotalHits is because this can change quite easily and without a good reason
            insta::with_settings!({
                info => &format!("{query}"),
                description => query.comment.unwrap_or_default(),
            }, {
                insta::assert_yaml_snapshot!(actual, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
            });
        }
    }

    #[test]
    fn serialize_address() {
        let response = NominatimAddressResponse {
            state: None,
            county: None,
            town: None,
            suburb: None,
            village: None,
            hamlet: None,
            road: None,
        };
        insta::assert_snapshot!(response.serialise(), @"")
    }
}
