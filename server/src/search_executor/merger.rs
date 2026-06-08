use std::collections::HashMap;

use meilisearch_sdk::search::SearchResult;

use super::ResultFacet;
use super::highlight::{HighlightContext, highlighted_name_for_hit};
use crate::external::meilisearch::{
    BUILDING_FACET, FACET_FIELD, GeoMSHit, LECTURE_FACET, LectureMSHit, MSHit, POI_FACET,
    ROOM_FACET, SITE_FACET,
};
use crate::routes::search::Limits;

pub(super) struct MergedSections {
    pub(super) sites: super::ResultsSection,
    pub(super) buildings: super::ResultsSection,
    pub(super) rooms: super::ResultsSection,
    pub(super) pois: super::ResultsSection,
    pub(super) lectures: super::ResultsSection,
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
    let mut lectures = empty_section(ResultFacet::Lectures, totals.lectures);
    let mut facet_order: Vec<ResultFacet> = Vec::with_capacity(5);

    // The visible count of any facet that already has hits is frozen the
    // moment a *new* facet's first hit appears in the ranking. This preserves
    // the original two-section behavior (later, lower-ranked hits don't
    // retroactively expand the default visible count of an earlier section).
    for hit in hits {
        let cap = active_count(&sites)
            + active_count(&buildings)
            + active_count(&rooms)
            + active_count(&pois)
            + active_count(&lectures);
        if cap >= limits.total_count {
            break;
        }

        // Each facet is its own hit variant, so the bucket is the variant. The
        // guard skips a hit whose section is already full; the catch-all then
        // drops it (`continue`) without ending the over-fetched ranking early.
        let (facet, entry) = match &hit.result {
            MSHit::Site(geo) if sites.entries.len() < limits.sites_count => (
                ResultFacet::Sites,
                make_building_like_entry(geo, hit, highlight),
            ),
            MSHit::Building(geo) if buildings.entries.len() < limits.buildings_count => (
                ResultFacet::Buildings,
                make_building_like_entry(geo, hit, highlight),
            ),
            MSHit::Room(geo) if rooms.entries.len() < limits.rooms_count => {
                (ResultFacet::Rooms, make_room_like_entry(geo, hit, highlight))
            }
            MSHit::Poi(geo) if pois.entries.len() < limits.pois_count => {
                (ResultFacet::Pois, make_room_like_entry(geo, hit, highlight))
            }
            MSHit::Lecture(lecture) if lectures.entries.len() < limits.lectures_count => (
                ResultFacet::Lectures,
                make_lecture_entry(lecture, hit, highlight),
            ),
            _ => continue,
        };

        if !facet_order.contains(&facet) {
            for prior in &facet_order {
                match prior {
                    ResultFacet::Sites => freeze_if_first(&mut sites),
                    ResultFacet::Buildings => freeze_if_first(&mut buildings),
                    ResultFacet::Rooms => freeze_if_first(&mut rooms),
                    ResultFacet::Pois => freeze_if_first(&mut pois),
                    ResultFacet::Lectures => freeze_if_first(&mut lectures),
                    ResultFacet::Addresses => {}
                }
            }
            facet_order.push(facet);
        }

        match facet {
            ResultFacet::Sites => sites.entries.push(entry),
            ResultFacet::Buildings => buildings.entries.push(entry),
            ResultFacet::Rooms => rooms.entries.push(entry),
            ResultFacet::Pois => pois.entries.push(entry),
            ResultFacet::Lectures => lectures.entries.push(entry),
            ResultFacet::Addresses => {}
        }
    }

    // Sections that never got their visible count frozen show all collected
    // entries by default.
    finalize_visible(&mut sites);
    finalize_visible(&mut buildings);
    finalize_visible(&mut rooms);
    finalize_visible(&mut pois);
    finalize_visible(&mut lectures);

    MergedSections {
        sites,
        buildings,
        rooms,
        pois,
        lectures,
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
    geo: &GeoMSHit,
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::ResultEntry {
    let name = highlighted_name_for_hit(hit, highlight);
    super::ResultEntry {
        id: geo.room_code.clone(),
        r#type: geo.r#type.clone(),
        subtext: geo.type_common_name.clone(),
        hit: hit.result.clone(),
        name,
        ..super::ResultEntry::default()
    }
}

fn make_room_like_entry(
    geo: &GeoMSHit,
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::ResultEntry {
    let name = highlighted_name_for_hit(hit, highlight);
    super::ResultEntry {
        id: geo.room_code.clone(),
        r#type: geo.r#type.clone(),
        subtext_bold: Some(geo.arch_name.clone().unwrap_or_default()),
        hit: hit.result.clone(),
        name,
        ..super::ResultEntry::default()
    }
}

/// Build a result entry for a lecture hit.
///
/// `subtext` carries the human `stp_type` label (consistent with how
/// building-like entries surface `type_common_name`), while the bilingual
/// titles and next occurrence are exposed as dedicated fields for clients that
/// render the richer lecture row.
fn make_lecture_entry(
    lecture: &LectureMSHit,
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::ResultEntry {
    let name = highlighted_name_for_hit(hit, highlight);
    super::ResultEntry {
        id: lecture.ms_id.clone(),
        r#type: lecture.r#type.clone(),
        subtext: lecture.type_common_name.clone(),
        name,
        title_de: Some(lecture.title_de.clone()),
        title_en: Some(lecture.title_en.clone()),
        next_occurrence_at: Some(lecture.next_occurrence_at),
        hit: hit.result.clone(),
        ..super::ResultEntry::default()
    }
}

#[derive(Default)]
struct FacetTotals {
    sites: usize,
    buildings: usize,
    rooms: usize,
    pois: usize,
    lectures: usize,
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
        lectures: facet.get(LECTURE_FACET).copied().unwrap_or(0),
    }
}
