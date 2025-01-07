use meilisearch_sdk::client::Client;
use parser::TextToken;
use serde::Serialize;
use std::fmt::{Debug, Formatter};
use tracing::error;

use crate::external::meilisearch::{GeoEntryQuery, MSHit};
use crate::external::nominatim::Nominatim;
use crate::limited::vec::LimitedVec;
use crate::routes::search::{Highlighting, Limits};
use crate::search_executor::parser::ParsedQuery;

mod formatter;
mod lexer;
mod merger;
mod parser;

#[derive(Serialize, Clone, Copy, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultFacet {
    SitesBuildings,
    Rooms,
    Addresses,
}

#[derive(Serialize, Clone, utoipa::ToSchema)]
pub struct ResultsSection {
    /// These indicate the type of item this represents
    pub(crate) facet: ResultFacet,
    entries: Vec<ResultEntry>,
    /// A recommendation how many of the entries should be displayed by default.
    ///
    /// The number is usually from `0`..`5`.
    /// More results might be displayed when clicking "expand".
    #[schema(example = 4)]
    n_visible: usize,
    /// The estimated (not exact) number of hits for that query
    #[serde(rename = "estimatedTotalHits")]
    #[schema(example = 6)]
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

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default, Debug, Clone, utoipa::ToSchema)]
struct ResultEntry {
    #[serde(skip)]
    hit: MSHit,
    /// The id of the location
    #[schema(example = "5510.03.002")]
    id: String,
    /// the type of the site/building
    #[schema(example = "room")]
    r#type: String,
    /// Subtext to show below the search result.
    ///
    /// Usually contains the context of where this rooms is located in.
    /// Currently not highlighted.
    #[schema(example = "5510.03.002 (\x19MW\x17 2001, Empore)")]
    name: String,
    /// Subtext to show below the search result.
    ///
    /// Usually contains the context of where this rooms is located in.
    /// Currently not highlighted.
    #[schema(example = "Maschinenwesen (MW)")]
    subtext: String,
    /// Subtext to show below the search (by default in bold and after the non-bold subtext).
    ///
    /// Usually contains the arch-id of the room, which is another common room id format, and supports highlighting.
    #[schema(example = "3002@5510")]
    subtext_bold: Option<String>,
    /// This is an optional feature, that is only supported for some rooms.
    ///
    /// It might be displayed instead or before the name, to show that a different room id format has matched, that was probably used.
    /// See the image below for an example.
    /// It will be cropped to a maximum length to not take too much space in UIs.
    /// Supports highlighting.
    parsed_id: Option<String>,
}

#[tracing::instrument]
pub async fn address_search(q: &str) -> LimitedVec<ResultsSection> {
    let results = match Nominatim::address_search(q).await {
        Ok(r) => r.0,
        Err(e) => {
            error!(error = ?e, "Error searching for addresses");
            return LimitedVec(vec![]);
        }
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

    let query = parsed_input
        .tokens
        .clone()
        .into_iter()
        .map(|s| match s {
            TextToken::Text(t) => t,
            TextToken::SplittableText((t1, t2)) => format!("{t1} {t2} {t1}{t2}"),
        })
        .collect::<Vec<String>>()
        .join(" ");
    let mut query = GeoEntryQuery::from((client, query, &limits, &highlighting));
    for sort in parsed_input.sorting.as_meilisearch_sorting() {
        query.with_sorting(sort);
    }
    if !parsed_input.filters.is_empty() {
        query.with_filtering(parsed_input.filters.as_meilisearch_filters());
    }

    let Ok(response) = query.execute().await else {
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
    use std::fmt::{Display, Formatter};

    use super::*;
    use crate::setup::tests::MeiliSearchTestContainer;

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
}
