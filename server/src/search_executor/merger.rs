use std::collections::HashMap;

use meilisearch_sdk::search::SearchResult;

use super::ResultFacet;
use super::highlight::{HighlightContext, highlighted_name_for_hit};
use crate::external::meilisearch::{
    BUILDING_FACET, EVENT_FACET, EventMSHit, FACET_FIELD, GeoMSHit, LECTURE_FACET, LectureMSHit,
    MSHit, POI_FACET, ROOM_FACET, SITE_FACET,
};
use crate::routes::search::Limits;

pub(super) struct MergedSections {
    pub(super) sites: super::LocationSection,
    pub(super) buildings: super::LocationSection,
    pub(super) rooms: super::LocationSection,
    pub(super) pois: super::LocationSection,
    pub(super) lectures: super::LectureSection,
    pub(super) events: super::EventSection,
    /// Facets in the order their first hit appeared in the ranked Meilisearch
    /// results. Facets that never received a hit are not included.
    pub(super) facet_order: Vec<ResultFacet>,
}

/// Shared handle over a section body's visible-count bookkeeping, so the
/// freeze/finalize helpers work uniformly across the location and lecture
/// section types.
trait SectionVisibility {
    fn entries_len(&self) -> usize;
    fn n_visible(&self) -> usize;
    fn set_n_visible(&mut self, n: usize);
}

impl SectionVisibility for super::LocationSection {
    fn entries_len(&self) -> usize {
        self.entries.len()
    }
    fn n_visible(&self) -> usize {
        self.n_visible
    }
    fn set_n_visible(&mut self, n: usize) {
        self.n_visible = n;
    }
}

impl SectionVisibility for super::LectureSection {
    fn entries_len(&self) -> usize {
        self.entries.len()
    }
    fn n_visible(&self) -> usize {
        self.n_visible
    }
    fn set_n_visible(&mut self, n: usize) {
        self.n_visible = n;
    }
}

impl SectionVisibility for super::EventSection {
    fn entries_len(&self) -> usize {
        self.entries.len()
    }
    fn n_visible(&self) -> usize {
        self.n_visible
    }
    fn set_n_visible(&mut self, n: usize) {
        self.n_visible = n;
    }
}

#[tracing::instrument(skip(hits, facet_distribution, highlight))]
pub(super) fn merge_search_results(
    limits: &Limits,
    hits: &[SearchResult<MSHit>],
    facet_distribution: Option<&HashMap<String, HashMap<String, usize>>>,
    highlight: &HighlightContext<'_>,
) -> MergedSections {
    let totals = facet_totals(facet_distribution);

    let mut sites = empty_location_section(totals.sites);
    let mut buildings = empty_location_section(totals.buildings);
    let mut rooms = empty_location_section(totals.rooms);
    let mut pois = empty_location_section(totals.pois);
    let mut lectures = super::LectureSection {
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits: totals.lectures,
    };
    let mut events = super::EventSection {
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits: totals.events,
    };
    let mut facet_order: Vec<ResultFacet> = Vec::with_capacity(6);

    // The visible count of any facet that already has hits is frozen the
    // moment a *new* facet's first hit appears in the ranking. This preserves
    // the original two-section behavior (later, lower-ranked hits don't
    // retroactively expand the default visible count of an earlier section).
    for hit in hits {
        let cap = active_count(&sites)
            + active_count(&buildings)
            + active_count(&rooms)
            + active_count(&pois)
            + active_count(&lectures)
            + active_count(&events);
        if cap >= limits.total_count {
            break;
        }

        // Each facet is its own hit variant, so the bucket is the variant, and
        // the entry is pushed straight into its concretely-typed section. The
        // guard skips a hit whose section is already full; the catch-all then
        // drops it (`continue`) without ending the over-fetched ranking early.
        // Pushing before the freeze bookkeeping is sound: freezing only touches
        // *prior* facets, never the one this hit lands in.
        let facet = match &hit.result {
            MSHit::Site(geo) if sites.entries.len() < limits.sites_count => {
                sites
                    .entries
                    .push(make_building_like_entry(geo, hit, highlight));
                ResultFacet::Sites
            }
            MSHit::Building(geo) if buildings.entries.len() < limits.buildings_count => {
                buildings
                    .entries
                    .push(make_building_like_entry(geo, hit, highlight));
                ResultFacet::Buildings
            }
            MSHit::Room(geo) if rooms.entries.len() < limits.rooms_count => {
                rooms
                    .entries
                    .push(make_room_like_entry(geo, hit, highlight));
                ResultFacet::Rooms
            }
            MSHit::Poi(geo) if pois.entries.len() < limits.pois_count => {
                pois.entries.push(make_room_like_entry(geo, hit, highlight));
                ResultFacet::Pois
            }
            MSHit::Lecture(lecture) if lectures.entries.len() < limits.lectures_count => {
                lectures
                    .entries
                    .push(make_lecture_entry(lecture, hit, highlight));
                ResultFacet::Lectures
            }
            MSHit::Event(event) if events.entries.len() < limits.events_count => {
                events.entries.push(make_event_entry(event, hit, highlight));
                ResultFacet::Events
            }
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
                    ResultFacet::Events => freeze_if_first(&mut events),
                    ResultFacet::Addresses => {}
                }
            }
            facet_order.push(facet);
        }
    }

    // Sections that never got their visible count frozen show all collected
    // entries by default.
    finalize_visible(&mut sites);
    finalize_visible(&mut buildings);
    finalize_visible(&mut rooms);
    finalize_visible(&mut pois);
    finalize_visible(&mut lectures);
    finalize_visible(&mut events);

    MergedSections {
        sites,
        buildings,
        rooms,
        pois,
        lectures,
        events,
        facet_order,
    }
}

/// Freeze the visible count of a higher-priority section the first time a
/// lower-priority hit lands. No-op if the section is empty (it stays at 0)
/// or already frozen.
fn freeze_if_first<S: SectionVisibility>(section: &mut S) {
    if section.n_visible() == 0 && section.entries_len() > 0 {
        section.set_n_visible(section.entries_len());
    }
}

fn finalize_visible<S: SectionVisibility>(section: &mut S) {
    if section.n_visible() == 0 {
        section.set_n_visible(section.entries_len());
    }
}

fn active_count<S: SectionVisibility>(section: &S) -> usize {
    if section.n_visible() == 0 {
        section.entries_len()
    } else {
        section.n_visible()
    }
}

fn empty_location_section(estimated_total_hits: usize) -> super::LocationSection {
    super::LocationSection {
        entries: Vec::new(),
        n_visible: 0,
        estimated_total_hits,
    }
}

fn make_building_like_entry(
    geo: &GeoMSHit,
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::LocationEntry {
    let name = highlighted_name_for_hit(hit, highlight);
    super::LocationEntry {
        id: geo.room_code.clone(),
        r#type: geo.r#type,
        subtext: geo.type_common_name.clone(),
        hit: Box::new(hit.result.clone()),
        name,
        subtext_bold: None,
        parsed_id: None,
    }
}

fn make_room_like_entry(
    geo: &GeoMSHit,
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::LocationEntry {
    let name = highlighted_name_for_hit(hit, highlight);
    super::LocationEntry {
        id: geo.room_code.clone(),
        r#type: geo.r#type,
        subtext: String::new(),
        subtext_bold: Some(geo.arch_name.clone().unwrap_or_default()),
        hit: Box::new(hit.result.clone()),
        name,
        parsed_id: None,
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
) -> super::LectureEntry {
    let name = highlighted_name_for_hit(hit, highlight);
    super::LectureEntry {
        id: lecture.ms_id.clone(),
        subtext: lecture.type_common_name.clone(),
        name,
        title_de: lecture.title_de.clone(),
        title_en: lecture.title_en.clone(),
        next_occurrence_at: lecture.next_occurrence_at,
        upcoming: lecture.upcoming.clone(),
    }
}

/// Build a result entry for an event hit.
///
/// Beyond the highlighted `name`, the entry is a 1:1 copy of the stored
/// document: every field is part of the proposal-form pre-fill payload.
fn make_event_entry(
    event: &EventMSHit,
    hit: &SearchResult<MSHit>,
    highlight: &HighlightContext<'_>,
) -> super::EventEntry {
    let name = highlighted_name_for_hit(hit, highlight);
    super::EventEntry {
        id: event.ms_id.clone(),
        name,
        description: event.description.clone(),
        starts_at: event.starts_at,
        ends_at: event.ends_at,
        lat: event.coords.lat,
        lon: event.coords.lng,
        organising_org_id: event.organising_org_id,
        image: event.image.clone(),
        image_author: event.image_author.clone(),
        image_thumb_offset: event.image_thumb_offset,
        image_header_offset: event.image_header_offset,
    }
}

#[derive(Default)]
struct FacetTotals {
    sites: usize,
    buildings: usize,
    rooms: usize,
    pois: usize,
    lectures: usize,
    events: usize,
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
        events: facet.get(EVENT_FACET).copied().unwrap_or(0),
    }
}
