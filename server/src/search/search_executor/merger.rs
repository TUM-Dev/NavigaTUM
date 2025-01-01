use meilisearch_sdk::search::{SearchResult, SearchResults};

use crate::external::meilisearch::MSHit;
use crate::search::search_executor::ResultFacet;

#[tracing::instrument(skip(merged_results, buildings_results, rooms_results))]
pub(super) fn merge_search_results(
    limits: &super::Limits,
    merged_results: &SearchResults<MSHit>,
    buildings_results: &SearchResults<MSHit>,
    rooms_results: &SearchResults<MSHit>,
) -> (super::ResultsSection, super::ResultsSection) {
    // First look up which buildings did match even with a closed query.
    // We can consider them more relevant.
    // TODO: This has to be implemented. closed_matching_buildings is not used further down in this function.
    let mut closed_matching_buildings = Vec::<String>::new();
    for hit in &buildings_results.hits {
        closed_matching_buildings.push(hit.result.id.clone());
    }

    let mut section_buildings = super::ResultsSection {
        facet: ResultFacet::SitesBuildings,
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits: buildings_results.estimated_total_hits.unwrap_or(0),
    };
    let mut section_rooms = super::ResultsSection {
        facet: ResultFacet::Rooms,
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits: rooms_results.estimated_total_hits.unwrap_or(0),
    };

    // TODO: Collapse joined buildings
    // let mut observed_joined_buildings = Vec::<String>::new();
    let mut observed_ids = Vec::<String>::new();
    for hits in [&merged_results.hits, &rooms_results.hits] {
        for hit in hits {
            // Prevent duplicates from being added to the results
            if observed_ids.contains(&hit.result.id) {
                continue;
            };
            observed_ids.push(hit.result.id.clone());

            // Total limit reached (does only count visible results)
            let current_buildings_cnt = if section_buildings.n_visible == 0 {
                section_buildings.entries.len()
            } else {
                section_buildings.n_visible
            };
            if section_rooms.entries.len() + current_buildings_cnt >= limits.total_count {
                break;
            }
            let formatted_name =
                extract_formatted_name(hit).unwrap_or_else(|| hit.result.name.clone());

            let hit = hit.result.clone();
            match hit.r#type.as_str() {
                "campus" | "site" | "area" | "building" | "joined_building" => {
                    if section_buildings.entries.len() < limits.buildings_count {
                        section_buildings.entries.push(super::ResultEntry {
                            hit: hit.clone(),
                            id: hit.id.to_string(),
                            r#type: hit.r#type,
                            name: formatted_name,
                            subtext: hit.type_common_name,
                            subtext_bold: None,
                            parsed_id: None,
                        });
                    }
                }
                "room" | "virtual_room" => {
                    if section_rooms.entries.len() < limits.rooms_count {
                        section_rooms.entries.push(super::ResultEntry {
                            hit: hit.clone(),
                            id: hit.id.to_string(),
                            r#type: hit.r#type,
                            name: formatted_name,
                            subtext_bold: Some(hit.arch_name.unwrap_or_default()),
                            ..super::ResultEntry::default()
                        });

                        // The first room in the results 'freezes' the number of visible buildings
                        if section_buildings.n_visible == 0 && section_rooms.entries.len() == 1 {
                            section_buildings.n_visible = section_buildings.entries.len();
                        }
                    }
                }
                _ => {}
            };
        }
    }
    section_rooms.n_visible = section_rooms.entries.len();

    (section_buildings, section_rooms)
}

fn extract_formatted_name(hit: &SearchResult<MSHit>) -> Option<String> {
    Some(
        hit.formatted_result
            .clone()? //I don't understand why this is needed, but the performance impact is minimal
            .get("name")?
            .as_str()?
            .to_string(),
    )
}
