use crate::search::search_executor::query::MSHit;
use meilisearch_sdk::search::{SearchResult, SearchResults};

pub(super) fn merge_search_results(
    args: &super::SanitisedSearchQueryArgs,
    res_merged: &SearchResults<MSHit>,
    res_buildings: &SearchResults<MSHit>,
    res_rooms: &SearchResults<MSHit>,
) -> (super::SearchResultsSection, super::SearchResultsSection) {
    // First look up which buildings did match even with a closed query.
    // We can consider them more relevant.
    // TODO: This has to be implemented. closed_matching_buildings is not used further down in this function.
    let mut closed_matching_buildings = Vec::<String>::new();
    for hit in &res_buildings.hits {
        closed_matching_buildings.push(hit.result.id.clone());
    }

    let mut section_buildings = super::SearchResultsSection {
        facet: "sites_buildings".to_string(),
        entries: Vec::new(),
        n_visible: None,
        estimated_total_hits: res_buildings.estimated_total_hits.unwrap_or(0),
    };
    let mut section_rooms = super::SearchResultsSection {
        facet: "rooms".to_string(),
        entries: Vec::new(),
        n_visible: None,
        estimated_total_hits: res_rooms.estimated_total_hits.unwrap_or(0),
    };

    // TODO: Collapse joined buildings
    // let mut observed_joined_buildings = Vec::<String>::new();
    let mut observed_ids = Vec::<String>::new();
    for hits in [&res_merged.hits, &res_rooms.hits] {
        for hit in hits.iter() {
            // Prevent duplicates from being added to the results
            if observed_ids.contains(&hit.result.id) {
                continue;
            };
            observed_ids.push(hit.result.id.clone());

            // Total limit reached (does only count visible results)
            let current_buildings_cnt = section_buildings
                .n_visible
                .unwrap_or(section_buildings.entries.len());
            if section_rooms.entries.len() + current_buildings_cnt >= args.limit_all {
                break;
            }
            let formatted_name =
                extract_formatted_name(hit).unwrap_or_else(|| hit.result.name.clone());

            let hit = hit.result.clone();
            match hit.r#type.as_str() {
                "campus" | "site" | "area" | "building" | "joined_building" => {
                    if section_buildings.entries.len() < args.limit_buildings {
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
                    if section_rooms.entries.len() < args.limit_rooms {
                        section_rooms.entries.push(super::ResultEntry {
                            hit: hit.clone(),
                            id: hit.id.to_string(),
                            r#type: hit.r#type,
                            name: formatted_name,
                            subtext_bold: Some(hit.arch_name.unwrap_or_default()),
                            ..super::ResultEntry::default()
                        });

                        // The first room in the results 'freezes' the number of visible buildings
                        if section_buildings.n_visible.is_none() && section_rooms.entries.len() == 1
                        {
                            section_buildings.n_visible = Some(section_buildings.entries.len());
                        }
                    }
                }
                _ => {}
            };
        }
    }

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
