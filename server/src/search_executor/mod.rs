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

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_cropping_full_shows_full_building_names() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        // Search with cropping enabled (default)
        let config_cropping = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_cropping = do_geoentry_search(
            &ms.client,
            "3002", // Room search that might have long building names
            Limits::default(),
            config_cropping,
        )
        .await;

        // Search with cropping disabled
        let config_no_cropping = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_no_cropping =
            do_geoentry_search(&ms.client, "3002", Limits::default(), config_no_cropping).await;

        // Find room entries
        let rooms_with_crop = results_cropping
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms));

        let rooms_without_crop = results_no_cropping
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms));

        if let (Some(with_crop), Some(without_crop)) = (rooms_with_crop, rooms_without_crop) {
            // Check if any parsed_ids contain ellipsis (indicating cropping)
            let has_ellipsis_with_crop = with_crop
                .entries
                .iter()
                .any(|e| e.parsed_id.as_ref().map_or(false, |p| p.contains("…")));

            let has_ellipsis_without_crop = without_crop
                .entries
                .iter()
                .any(|e| e.parsed_id.as_ref().map_or(false, |p| p.contains("…")));

            // If long building names exist (indicated by ellipsis when cropping is enabled)
            // then cropping should be disabled when the flag is set
            if has_ellipsis_with_crop {
                assert!(
                    !has_ellipsis_without_crop,
                    "Building names should NOT be cropped when cropping=full"
                );
            }
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_both_flags_work_together() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        // Both features disabled
        let config = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Roomfinder,
        };

        let results = do_geoentry_search(&ms.client, "1010", Limits::default(), config).await;

        let rooms = results
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms));

        if let Some(room_section) = rooms {
            for entry in &room_section.entries {
                // Check that parsed_id matches subtext_bold (which is archname@building_id)
                assert_eq!(
                    entry.parsed_id, entry.subtext_bold,
                    "parsed_id should be archname@building_id when parsed_id=roomfinder"
                );
            }
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_custom_highlighting_with_formatting() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        // Custom HTML highlighting
        let config = FormattingConfig {
            highlighting: Highlighting {
                pre: "<em>".to_string(),
                post: "</em>".to_string(),
            },
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results =
            do_geoentry_search(&ms.client, "mi560602018", Limits::default(), config).await;

        let rooms = results
            .0
            .iter()
            .find(|s| matches!(s.facet, ResultFacet::Rooms));

        if let Some(room_section) = rooms {
            // Check if custom highlighting appears in parsed_id or name
            let has_custom_highlighting = room_section.entries.iter().any(|e| {
                let in_parsed_id = e
                    .parsed_id
                    .as_ref()
                    .map_or(false, |p| p.contains("<em>") || p.contains("</em>"));

                let in_name = e.name.contains("<em>") || e.name.contains("</em>");

                in_parsed_id || in_name
            });

            // If MI rooms are found, some highlighting should be present
            if !room_section.entries.is_empty() {
                assert!(
                    has_custom_highlighting,
                    "Custom highlighting markers should appear in results"
                );
            }
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_building_formats() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        let config = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let config_no_prefix = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Roomfinder,
        };

        // Test different building formats
        let test_cases = vec![
            ("mw1801", "MW"),       // Maschinenwesen
            ("mi560602018", "MI"),  // Mathematik/Informatik
            ("pi510101294", "PH"),  // Physik
            ("ch5406EG600B", "CH"), // Chemie
        ];

        for (query, expected_prefix) in test_cases {
            let results =
                do_geoentry_search(&ms.client, query, Limits::default(), config.clone()).await;

            let results_no_prefix = do_geoentry_search(
                &ms.client,
                query,
                Limits::default(),
                config_no_prefix.clone(),
            )
            .await;

            let rooms = results
                .0
                .iter()
                .find(|s| matches!(s.facet, ResultFacet::Rooms));

            let rooms_no_prefix = results_no_prefix
                .0
                .iter()
                .find(|s| matches!(s.facet, ResultFacet::Rooms));

            if let Some(room_section) = rooms {
                // If rooms are found for this building, check for the prefix
                if !room_section.entries.is_empty() {
                    let has_prefix = room_section.entries.iter().any(|e| {
                        e.parsed_id
                            .as_ref()
                            .map_or(false, |p| p.contains(expected_prefix))
                    });

                    assert!(
                        has_prefix,
                        "Entries for query '{}' should contain prefix '{}'",
                        query, expected_prefix
                    );
                }
            }

            if let Some(room_section_no_prefix) = rooms_no_prefix {
                // If rooms are found for this building, check that no prefix is present
                if !room_section_no_prefix.entries.is_empty() {
                    let has_prefix = room_section_no_prefix.entries.iter().any(|e| {
                        e.parsed_id
                            .as_ref()
                            .map_or(false, |p| p.contains(expected_prefix))
                    });

                    assert!(
                        !has_prefix,
                        "Entries for query '{}' should NOT contain prefix '{}' when parsed_id=roomfinder",
                        query, expected_prefix
                    );

                    // Check that the raw Roomfinder format (archname@building_id) is used
                    let has_raw_archname_format = room_section_no_prefix
                        .entries
                        .iter()
                        .any(|e| e.parsed_id.as_ref().map_or(false, |p| p.contains('@')));

                    assert!(
                        has_raw_archname_format,
                        "Entries for query '{}' should contain the raw Roomfinder format (i.e., '@' symbol) when parsed_id=roomfinder",
                        query
                    );
                }
            }
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_cropping_behavior_for_1010() {
        let ms = MeiliSearchTestContainer::new().await;
        crate::setup::meilisearch::load_data(&ms.client)
            .await
            .unwrap();

        let query = "1010";

        // 1. Search with default settings (Cropping Enabled)
        let config_default = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Crop,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_cropped =
            do_geoentry_search(&ms.client, query, Limits::default(), config_default).await;

        // 2. Search with cropping disabled
        let config_no_crop = FormattingConfig {
            highlighting: Highlighting::default(),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Prefixed,
        };

        let results_full =
            do_geoentry_search(&ms.client, query, Limits::default(), config_no_crop).await;

        // Helper to extract parsed_ids from the Rooms section
        let get_ids = |res: &LimitedVec<ResultsSection>| -> Vec<String> {
            res.0
                .iter()
                .find(|s| matches!(s.facet, ResultFacet::Rooms))
                .map(|s| {
                    s.entries
                        .iter()
                        .filter_map(|e| e.parsed_id.clone())
                        .collect()
                })
                .unwrap_or_default()
        };

        let ids_cropped = get_ids(&results_cropped);
        let ids_full = get_ids(&results_full);

        // Assertions
        assert!(
            !ids_cropped.is_empty(),
            "Query '1010' should return results"
        );

        // Verify that the default behavior crops the long building name associated with 1010
        // (Assuming 1010 belongs to the main building which usually has a long name)
        let has_ellipsis = ids_cropped.iter().any(|id| id.contains('…'));
        assert!(
            has_ellipsis,
            "Default behavior for '1010' should include cropping (ellipsis)"
        );

        // Verify that the specific flag disables this cropping
        let has_ellipsis_full = ids_full.iter().any(|id| id.contains('…'));
        assert!(
            !has_ellipsis_full,
            "cropping=full should return full names without ellipses"
        );

        // Verify the IDs are actually different (one is shorter than the other)
        // This confirms the flag actually changed the output
        assert_ne!(
            ids_cropped, ids_full,
            "Results should differ based on cropping flag"
        );
    }
}
