use std::collections::HashMap;

use meilisearch_sdk::search::SearchResult;

use super::ResultFacet;
use crate::external::meilisearch::{
    BUILDING_FACET, FACET_FIELD, MSHit, POI_FACET, ROOM_FACET, SITE_FACET,
};
use crate::routes::search::Limits;

pub(super) struct MergedSections {
    pub(super) sites: super::ResultsSection,
    pub(super) buildings: super::ResultsSection,
    pub(super) rooms: super::ResultsSection,
    pub(super) pois: super::ResultsSection,
}

#[tracing::instrument(skip(hits, facet_distribution))]
pub(super) fn merge_search_results(
    limits: &Limits,
    hits: &[SearchResult<MSHit>],
    facet_distribution: Option<&HashMap<String, HashMap<String, usize>>>,
) -> MergedSections {
    let totals = facet_totals(facet_distribution);

    let mut sites = empty_section(ResultFacet::Sites, totals.sites);
    let mut buildings = empty_section(ResultFacet::Buildings, totals.buildings);
    let mut rooms = empty_section(ResultFacet::Rooms, totals.rooms);
    let mut pois = empty_section(ResultFacet::Pois, totals.pois);

    // Visible counts of higher-priority facets are frozen the moment a
    // lower-priority facet's first hit appears in the ranking. This preserves
    // the original two-section behavior (a lower-ranked room would not
    // retroactively expand the default visible buildings count) and
    // generalizes it to four facets: sites > buildings > rooms > pois.
    for hit in hits {
        let cap = active_count(&sites) + active_count(&buildings) + active_count(&rooms) + active_count(&pois);
        if cap >= limits.total_count {
            break;
        }

        match hit.result.facet.as_deref() {
            Some(SITE_FACET) if sites.entries.len() < limits.sites_count => {
                sites.entries.push(make_building_like_entry(hit));
            }
            Some(BUILDING_FACET) if buildings.entries.len() < limits.buildings_count => {
                freeze_if_first(&mut sites);
                buildings.entries.push(make_building_like_entry(hit));
            }
            Some(ROOM_FACET) if rooms.entries.len() < limits.rooms_count => {
                freeze_if_first(&mut sites);
                freeze_if_first(&mut buildings);
                rooms.entries.push(make_room_like_entry(hit));
            }
            Some(POI_FACET) if pois.entries.len() < limits.pois_count => {
                freeze_if_first(&mut sites);
                freeze_if_first(&mut buildings);
                freeze_if_first(&mut rooms);
                pois.entries.push(make_room_like_entry(hit));
            }
            _ => {}
        };
    }

    // Sections that never got their visible count frozen show all collected
    // entries by default.
    finalize_visible(&mut sites);
    finalize_visible(&mut buildings);
    finalize_visible(&mut rooms);
    finalize_visible(&mut pois);

    MergedSections {
        sites,
        buildings,
        rooms,
        pois,
    }
}

/// Freeze the visible count of a higher-priority section the first time a
/// lower-priority hit lands. No-op if the section is empty (it stays at 0)
/// or already frozen.
fn freeze_if_first(section: &mut super::ResultsSection) {
    if section.n_visible == 0 && !section.entries.is_empty() {
        section.n_visible = section.entries.len();
    }
}

fn finalize_visible(section: &mut super::ResultsSection) {
    if section.n_visible == 0 {
        section.n_visible = section.entries.len();
    }
}

fn active_count(section: &super::ResultsSection) -> usize {
    if section.n_visible == 0 {
        section.entries.len()
    } else {
        section.n_visible
    }
}

fn empty_section(facet: ResultFacet, estimated_total_hits: usize) -> super::ResultsSection {
    super::ResultsSection {
        facet,
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits,
    }
}

fn make_building_like_entry(hit: &SearchResult<MSHit>) -> super::ResultEntry {
    let result = hit.result.clone();
    let name = extract_formatted_name(hit).unwrap_or_else(|| result.name.clone());
    super::ResultEntry {
        id: result.room_code.clone(),
        r#type: result.r#type.clone(),
        subtext: result.type_common_name.clone(),
        hit: result,
        name,
        subtext_bold: None,
        parsed_id: None,
    }
}

fn make_room_like_entry(hit: &SearchResult<MSHit>) -> super::ResultEntry {
    let result = hit.result.clone();
    let name = extract_formatted_name(hit).unwrap_or_else(|| result.name.clone());
    super::ResultEntry {
        id: result.room_code.clone(),
        r#type: result.r#type.clone(),
        subtext_bold: Some(result.arch_name.clone().unwrap_or_default()),
        hit: result,
        name,
        ..super::ResultEntry::default()
    }
}

#[derive(Default)]
struct FacetTotals {
    sites: usize,
    buildings: usize,
    rooms: usize,
    pois: usize,
}

fn facet_totals(distribution: Option<&HashMap<String, HashMap<String, usize>>>) -> FacetTotals {
    let Some(facet) = distribution.and_then(|d| d.get(FACET_FIELD)) else {
        return FacetTotals::default();
    };
    FacetTotals {
        sites: facet.get(SITE_FACET).copied().unwrap_or(0),
        buildings: facet.get(BUILDING_FACET).copied().unwrap_or(0),
        rooms: facet.get(ROOM_FACET).copied().unwrap_or(0),
        pois: facet.get(POI_FACET).copied().unwrap_or(0),
    }
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
