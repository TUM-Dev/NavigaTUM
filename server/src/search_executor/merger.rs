use std::collections::HashMap;

use meilisearch_sdk::search::SearchResult;

use super::ResultFacet;
use crate::external::meilisearch::{BUILDING_FACET, FACET_FIELD, MSHit, ROOM_FACET};
use crate::routes::search::Limits;

#[tracing::instrument(skip(hits, facet_distribution))]
pub(super) fn merge_search_results(
    limits: &Limits,
    hits: &[SearchResult<MSHit>],
    facet_distribution: Option<&HashMap<String, HashMap<String, usize>>>,
) -> (super::ResultsSection, super::ResultsSection) {
    let (buildings_total, rooms_total) = facet_totals(facet_distribution);

    let mut section_buildings = super::ResultsSection {
        facet: ResultFacet::SitesBuildings,
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits: buildings_total,
    };
    let mut section_rooms = super::ResultsSection {
        facet: ResultFacet::Rooms,
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits: rooms_total,
    };

    for hit in hits {
        let current_buildings_cnt = if section_buildings.n_visible == 0 {
            section_buildings.entries.len()
        } else {
            section_buildings.n_visible
        };
        if section_rooms.entries.len() + current_buildings_cnt >= limits.total_count {
            break;
        }

        match hit.result.r#type.as_str() {
            "campus" | "site" | "area" | "building" | "joined_building"
                if section_buildings.entries.len() < limits.buildings_count =>
            {
                let result = hit.result.clone();
                let name = extract_formatted_name(hit).unwrap_or_else(|| result.name.clone());
                section_buildings.entries.push(super::ResultEntry {
                    id: result.room_code.clone(),
                    r#type: result.r#type.clone(),
                    subtext: result.type_common_name.clone(),
                    hit: result,
                    name,
                    subtext_bold: None,
                    parsed_id: None,
                });
            }
            "room" | "virtual_room" | "poi" if section_rooms.entries.len() < limits.rooms_count => {
                let result = hit.result.clone();
                let name = extract_formatted_name(hit).unwrap_or_else(|| result.name.clone());
                section_rooms.entries.push(super::ResultEntry {
                    id: result.room_code.clone(),
                    r#type: result.r#type.clone(),
                    subtext_bold: Some(result.arch_name.clone().unwrap_or_default()),
                    hit: result,
                    name,
                    ..super::ResultEntry::default()
                });

                // The first room in the results 'freezes' the number of visible
                // buildings: a hit appearing after rooms in the ranking should
                // not retroactively expand the default building section.
                if section_buildings.n_visible == 0 && section_rooms.entries.len() == 1 {
                    section_buildings.n_visible = section_buildings.entries.len();
                }
            }
            _ => {}
        };
    }
    section_rooms.n_visible = section_rooms.entries.len();

    (section_buildings, section_rooms)
}

fn facet_totals(distribution: Option<&HashMap<String, HashMap<String, usize>>>) -> (usize, usize) {
    let Some(facet) = distribution.and_then(|d| d.get(FACET_FIELD)) else {
        return (0, 0);
    };
    let buildings = facet.get(BUILDING_FACET).copied().unwrap_or(0);
    let rooms = facet.get(ROOM_FACET).copied().unwrap_or(0);
    (buildings, rooms)
}

fn extract_formatted_name(hit: &SearchResult<MSHit>) -> Option<String> {
    Some(
        hit.formatted_result
            .as_ref()?
            .get("name")?
            .as_str()?
            .to_string(),
    )
}
