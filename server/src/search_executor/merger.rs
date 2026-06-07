use std::collections::HashMap;

use meilisearch_sdk::search::SearchResult;

use super::ResultFacet;
use super::highlight::{HighlightContext, highlighted_name_for_hit};
use crate::external::meilisearch::{
    BUILDING_FACET, FACET_FIELD, MSHit, POI_FACET, ROOM_FACET, SITE_FACET,
};
use crate::routes::search::Limits;

pub(super) struct MergedSections {
    pub(super) sites: super::ResultsSection,
    pub(super) buildings: super::ResultsSection,
    pub(super) rooms: super::ResultsSection,
    pub(super) pois: super::ResultsSection,
    /// Facets in the order their first hit appeared in the ranked Meilisearch
    /// results. Facets that never received a hit are not included.
    pub(super) facet_order: Vec<ResultFacet>,
}

#[tracing::instrument(skip(hits, facet_distribution, highlight))]
pub(super) fn merge_search_results(
    limits: &Limits,
    hits: &[SearchResult<MSHit>],
    facet_distribution: Option<&HashMap<String, HashMap<String, usize>>>,
    highlight: &HighlightContext<'_>,
) -> MergedSections {
    let totals = facet_totals(facet_distribution);

    let mut sites = empty_section(ResultFacet::Sites, totals.sites);
    let mut buildings = empty_section(ResultFacet::Buildings, totals.buildings);
    let mut rooms = empty_section(ResultFacet::Rooms, totals.rooms);
    let mut pois = empty_section(ResultFacet::Pois, totals.pois);
    let mut facet_order: Vec<ResultFacet> = Vec::with_capacity(4);

    // The visible count of any facet that already has hits is frozen the
    // moment a *new* facet's first hit appears in the ranking. This preserves
    // the original two-section behavior (later, lower-ranked hits don't
    // retroactively expand the default visible count of an earlier section).
    for hit in hits {
        let cap = active_count(&sites)
            + active_count(&buildings)
            + active_count(&rooms)
            + active_count(&pois);
        if cap >= limits.total_count {
            break;
        }

        let facet = match hit.result.facet.as_deref() {
            Some(SITE_FACET) if sites.entries.len() < limits.sites_count => ResultFacet::Sites,
            Some(BUILDING_FACET) if buildings.entries.len() < limits.buildings_count => {
                ResultFacet::Buildings
            }
            Some(ROOM_FACET) if rooms.entries.len() < limits.rooms_count => ResultFacet::Rooms,
            Some(POI_FACET) if pois.entries.len() < limits.pois_count => ResultFacet::Pois,
            _ => continue,
        };

        if !facet_order.contains(&facet) {
            for prior in &facet_order {
                match prior {
                    ResultFacet::Sites => freeze_if_first(&mut sites),
                    ResultFacet::Buildings => freeze_if_first(&mut buildings),
                    ResultFacet::Rooms => freeze_if_first(&mut rooms),
                    ResultFacet::Pois => freeze_if_first(&mut pois),
                    ResultFacet::Addresses => {}
                }
            }
            facet_order.push(facet);
        }

        match facet {
            ResultFacet::Sites => sites.entries.push(make_building_like_entry(hit, highlight)),
            ResultFacet::Buildings => {
                buildings
                    .entries
                    .push(make_building_like_entry(hit, highlight));
            }
            ResultFacet::Rooms => rooms.entries.push(make_room_like_entry(hit, highlight)),
            ResultFacet::Pois => pois.entries.push(make_room_like_entry(hit, highlight)),
            ResultFacet::Addresses => {}
        }
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
        facet_order,
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

fn make_building_like_entry(
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::ResultEntry {
    let result = hit.result.clone();
    let name = highlighted_name_for_hit(hit, highlight);
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

fn make_room_like_entry(
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::ResultEntry {
    let result = hit.result.clone();
    let name = highlighted_name_for_hit(hit, highlight);
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
