use meilisearch_sdk::client::Client;
use parser::TextToken;
use serde::Serialize;
use std::fmt::{Debug, Formatter};
use tracing::error;

use crate::external::meilisearch::{GeoEntryQuery, MSHit};
use crate::external::nominatim::Nominatim;
use crate::limited::vec::LimitedVec;
use crate::routes::search::{FormattingConfig, Limits};
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
    limits: Limits,
    formatting_config: FormattingConfig,
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
    let mut query = GeoEntryQuery::from((client, query, &limits, &formatting_config));
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
    let visitor = formatter::RoomVisitor::from((parsed_input, formatting_config));
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
    use crate::routes::search::{CroppingMode, Highlighting, ParsedIdMode};
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
                Limits::default(),
                FormattingConfig::default(),
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

            // Fail fast: a "good" query must return something; empty results usually mean a broken index/search.
            assert!(
                !actual.is_empty(),
                "{query}\n\
                Expected at least one results section, but got none"
            );

            assert!(
                query.actual_matches_among(&actual),
                "{query}\n\
                Since it can't, please move it to .bad list"
            );

            // Redact `estimatedTotalHits` to reduce snapshot churn.
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

            // "Bad" queries may return results, but must not surface the `target` in the first `among` results.
            assert!(
                !query.actual_matches_among(&actual),
                "{query}\n\
                Since it can, please move it to .good list"
            );

            // Redact `estimatedTotalHits` to reduce snapshot churn.
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
    async fn test_cropping_full_shows_full_building_names() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        // Verify `CroppingMode` changes output; fail fast on missing Rooms/entries and snapshot both variants.
        let query = "N-1406";

        // Search with cropping enabled.
        let config_cropping = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_cropping =
            do_geoentry_search(&ms.client, query, Limits::default(), config_cropping).await;

        // Search with cropping disabled.
        let config_no_cropping = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_no_cropping =
            do_geoentry_search(&ms.client, query, Limits::default(), config_no_cropping).await;

        // Extract Rooms; fail fast to avoid silent no-ops.
        let rooms_with_crop = results_cropping
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms))
            .expect("Expected a Rooms section for cropping=CROP test query");

        let rooms_without_crop = results_no_cropping
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms))
            .expect("Expected a Rooms section for cropping=FULL test query");

        assert!(
            !rooms_with_crop.entries.is_empty(),
            "Expected at least one room entry for cropping=CROP test query"
        );
        assert!(
            !rooms_without_crop.entries.is_empty(),
            "Expected at least one room entry for cropping=FULL test query"
        );

        // Compare deterministically by sorting by `id` so ranking changes don't flap.
        let mut ids_cropped: Vec<(String, Option<String>)> = rooms_with_crop
            .entries
            .iter()
            .map(|e| (e.id.clone(), e.parsed_id.clone()))
            .collect();
        ids_cropped.sort_by(|a, b| a.0.cmp(&b.0));

        let mut ids_full: Vec<(String, Option<String>)> = rooms_without_crop
            .entries
            .iter()
            .map(|e| (e.id.clone(), e.parsed_id.clone()))
            .collect();
        ids_full.sort_by(|a, b| a.0.cmp(&b.0));

        // For the same `id`, `cropping=FULL` must not produce a shorter `parsed_id` than `cropping=CROP`.
        for ((id_c, pid_c), (id_f, pid_f)) in ids_cropped.iter().zip(ids_full.iter()) {
            assert_eq!(id_c, id_f, "Expected comparable entries when sorting by id");
            if let (Some(c), Some(f)) = (pid_c.as_ref(), pid_f.as_ref()) {
                assert!(
                    f.chars().count() >= c.chars().count(),
                    "Expected cropping=FULL parsed_id to be >= cropping=CROP parsed_id length for id={}",
                    id_c
                );
            }
        }

        // Snapshot both outputs; redact `estimatedTotalHits`.
        insta::with_settings!({
            info => &"cropping=crop",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_cropping.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });

        insta::with_settings!({
            info => &"cropping=full",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_no_cropping.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_both_flags_work_together() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        // Validate `ParsedIdMode::Roomfinder`: `parsed_id` should look like an arch id (contains '@'); snapshot output.
        let config = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Roomfinder,
        };

        // Use a canonical query from the good list to avoid accidental no-op.
        let results = do_geoentry_search(&ms.client, "N-1406", Limits::default(), config).await;

        let room_section = results
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms))
            .expect("Expected a Rooms section for Roomfinder mode test");

        assert!(
            !room_section.entries.is_empty(),
            "Expected at least one room entry for Roomfinder mode test"
        );

        for entry in &room_section.entries {
            let pid = entry
                .parsed_id
                .as_ref()
                .expect("Expected parsed_id to be present in Roomfinder mode");

            assert!(
                pid.contains('@'),
                "Expected Roomfinder parsed_id to contain '@' (arch_id@building_id), got: {}",
                pid
            );
        }

        insta::with_settings!({
            info => &"parsed_id=roomfinder,cropping=full",
            description => "Query: N-1406",
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_custom_highlighting_with_formatting() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        // Validate custom highlighting end-to-end; "MW1801" is a canonical query that tends to trigger highlighting.
        let query = "MW1801";

        let config = FormattingConfig {
            highlighting: Highlighting {
                pre: "<em>".to_string(),
                post: "</em>".to_string(),
            },
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results = do_geoentry_search(&ms.client, query, Limits::default(), config).await;

        let room_section = results
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms))
            .expect("Expected a Rooms section for highlighting test");

        assert!(
            !room_section.entries.is_empty(),
            "Expected at least one room entry for highlighting test"
        );

        let has_custom_highlighting = room_section.entries.iter().any(|e| {
            let in_parsed_id = e
                .parsed_id
                .as_ref()
                .map_or(false, |p| p.contains("<em>") || p.contains("</em>"));

            let in_name = e.name.contains("<em>") || e.name.contains("</em>");

            in_parsed_id || in_name
        });

        assert!(
            has_custom_highlighting,
            "Expected custom highlighting markers to appear in results for query '{}'",
            query
        );

        insta::with_settings!({
            info => &"highlighting=<em>",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_building_formats() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        let config_prefixed = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let config_roomfinder = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Roomfinder,
        };

        // Canonical queries that exercise prefix selection; use `starts_with` to avoid substring false positives.
        let test_cases = vec![
            ("MW1801", "MW "),     // Maschinenwesen (splitting necessary)
            ("MI HS 3", "MI "),    // Mathematik/Informatik
            ("342 Physik", "PH "), // Physik
        ];

        for (query, expected_prefix) in test_cases {
            let results_prefixed = do_geoentry_search(
                &ms.client,
                query,
                Limits::default(),
                config_prefixed.clone(),
            )
            .await;

            let results_roomfinder = do_geoentry_search(
                &ms.client,
                query,
                Limits::default(),
                config_roomfinder.clone(),
            )
            .await;

            let rooms_prefixed = results_prefixed
                .0
                .iter()
                .find(|s| matches!(s.facet, ResultFacet::Rooms))
                .expect("Expected a Rooms section for prefixed mode");

            assert!(
                !rooms_prefixed.entries.is_empty(),
                "Expected at least one room entry for prefixed mode query '{}'",
                query
            );

            let has_expected_prefix = rooms_prefixed.entries.iter().any(|e| {
                e.parsed_id
                    .as_ref()
                    .map_or(false, |p| p.starts_with(expected_prefix))
            });

            assert!(
                has_expected_prefix,
                "Expected at least one parsed_id to start with '{}' for query '{}'",
                expected_prefix, query
            );

            let rooms_roomfinder = results_roomfinder
                .0
                .iter()
                .find(|s| matches!(s.facet, ResultFacet::Rooms))
                .expect("Expected a Rooms section for roomfinder mode");

            assert!(
                !rooms_roomfinder.entries.is_empty(),
                "Expected at least one room entry for roomfinder mode query '{}'",
                query
            );

            let has_any_prefix = rooms_roomfinder.entries.iter().any(|e| {
                e.parsed_id
                    .as_ref()
                    .map_or(false, |p| p.starts_with(expected_prefix))
            });

            assert!(
                !has_any_prefix,
                "Expected no parsed_id to start with '{}' when parsed_id=roomfinder for query '{}'",
                expected_prefix, query
            );

            let has_raw_archname_format = rooms_roomfinder
                .entries
                .iter()
                .any(|e| e.parsed_id.as_ref().map_or(false, |p| p.contains('@')));

            assert!(
                has_raw_archname_format,
                "Expected at least one Roomfinder parsed_id to contain '@' for query '{}'",
                query
            );

            insta::with_settings!({
                info => &"parsed_id=prefixed",
                description => format!("Query: {query}"),
            }, {
                insta::assert_yaml_snapshot!(results_prefixed.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
            });

            insta::with_settings!({
                info => &"parsed_id=roomfinder",
                description => format!("Query: {query}"),
            }, {
                insta::assert_yaml_snapshot!(results_roomfinder.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
            });
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_cropping_behavior_for_1010() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        // Verify `CroppingMode` changes output; compare `parsed_id` lengths by sorted `id` (Full must not be shorter).
        let query = "1010 znn";

        // 1. Search with cropping enabled
        let config_cropped = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_cropped =
            do_geoentry_search(&ms.client, query, Limits::default(), config_cropped).await;

        // 2. Search with cropping disabled
        let config_full = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_full =
            do_geoentry_search(&ms.client, query, Limits::default(), config_full).await;

        let rooms_cropped = results_cropped
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms))
            .expect("Expected a Rooms section for cropping=CROP");

        let rooms_full = results_full
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms))
            .expect("Expected a Rooms section for cropping=FULL");

        assert!(
            !rooms_cropped.entries.is_empty(),
            "Expected at least one room entry for cropping=CROP query '{}'",
            query
        );
        assert!(
            !rooms_full.entries.is_empty(),
            "Expected at least one room entry for cropping=FULL query '{}'",
            query
        );

        // Deterministic comparison: sort by id and compare overlapping entries.
        let mut cropped: Vec<(String, Option<String>)> = rooms_cropped
            .entries
            .iter()
            .map(|e| (e.id.clone(), e.parsed_id.clone()))
            .collect();
        cropped.sort_by(|a, b| a.0.cmp(&b.0));

        let mut full: Vec<(String, Option<String>)> = rooms_full
            .entries
            .iter()
            .map(|e| (e.id.clone(), e.parsed_id.clone()))
            .collect();
        full.sort_by(|a, b| a.0.cmp(&b.0));

        // Assert FULL is never shorter than CROP for the same `id`.
        for ((id_c, pid_c), (id_f, pid_f)) in cropped.iter().zip(full.iter()) {
            assert_eq!(id_c, id_f, "Expected comparable entries when sorting by id");
            if let (Some(c), Some(f)) = (pid_c.as_ref(), pid_f.as_ref()) {
                assert!(
                    f.chars().count() >= c.chars().count(),
                    "Expected cropping=FULL parsed_id to be >= cropping=CROP parsed_id length for id={}",
                    id_c
                );
            }
        }

        insta::with_settings!({
            info => &"cropping=crop",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_cropped.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });

        insta::with_settings!({
            info => &"cropping=full",
            description => format!("Query: {query}"),
        }, {
            insta::assert_yaml_snapshot!(results_full.0, { ".**.estimatedTotalHits" => "[estimatedTotalHits]"});
        });
    }
}
