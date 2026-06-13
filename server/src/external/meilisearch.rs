use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use meilisearch_sdk::errors::Error;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::{
    FederatedMultiSearchResponse, FederationOptions, MergeFacets, SearchQuery, Selectors,
};
use serde::{Deserialize, Serialize};

use crate::routes::search::{FormattingConfig, Limits};

pub(crate) const ENTRIES_INDEX: &str = "entries";
pub(crate) const FACET_FIELD: &str = "facet";
pub(crate) const SITE_FACET: &str = "site";
pub(crate) const BUILDING_FACET: &str = "building";
pub(crate) const ROOM_FACET: &str = "room";
pub(crate) const POI_FACET: &str = "poi";
pub(crate) const LECTURE_FACET: &str = "lecture";
pub(crate) const EVENT_FACET: &str = "event";
/// Ordered list of facets the search federates over. Order is the priority
/// used by the merger when distributing the over-fetched federation budget
/// across facet caps. Lectures trail the geo facets so a course title only
/// surfaces once the geo entries it competes with have had their say; events
/// trail last as the only default-disabled facet.
pub(crate) const FACETS: &[&str] = &[
    SITE_FACET,
    BUILDING_FACET,
    ROOM_FACET,
    POI_FACET,
    LECTURE_FACET,
    EVENT_FACET,
];

/// Allowlisted values for the `?type=` query parameter.
///
/// Modeled as an enum so `serde` rejects unknown values with a 400 instead of
/// silently dropping them, and so the `OpenAPI` schema advertises the exact set
/// of accepted values.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FacetFilter {
    Site,
    Building,
    Room,
    Poi,
    Lecture,
    /// Requesting the default-disabled event facet implies enabling it.
    Event,
}

impl FacetFilter {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Site => SITE_FACET,
            Self::Building => BUILDING_FACET,
            Self::Room => ROOM_FACET,
            Self::Poi => POI_FACET,
            Self::Lecture => LECTURE_FACET,
            Self::Event => EVENT_FACET,
        }
    }
}

/// Federation has no per-facet quota - it merges by `_rankingScore` only. We
/// over-fetch by this factor so the merger downstream can still fill all
/// per-facet caps when one facet dominates the ranking (observed: queries
/// like `MW1801` returned 14 buildings + 1 room at the natural cap of 15).
/// Empirical; revisit if facet-starvation shows up in metrics.
const FEDERATION_OVERFETCH_FACTOR: usize = 4;

/// Per-query federation weight for the lecture facet. Meilisearch multiplies
/// each hit's `_rankingScore` by its query weight before merging the federated
/// results, so a value below `1.0` softly demotes lecture hits beneath
/// equally-strong geo matches: a same-strength title match on a lecture loses
/// to a same-strength match on a room or building. It is a soft constraint, not
/// a hard pin - an exact-title lecture match can still outrank a weak room
/// match. Empirical; revisit if snapshots show the deprioritisation is too
/// aggressive or too weak.
const LECTURE_FEDERATION_WEIGHT: f32 = 0.5;

/// Per-query federation weight for the event facet. Same rationale as
/// [`LECTURE_FEDERATION_WEIGHT`]: an event is a non-geo identity that should
/// lose against equally-strong location matches, not a hard pin below them.
const EVENT_FEDERATION_WEIGHT: f32 = 0.5;

/// The type of a `NavigaTUM` entity surfaced as a search result.
///
/// The closed set of location types the data pipeline exports (`valid_types`
/// in `data/processors/schema.py`, minus the synthetic, non-searchable
/// `root`). Every variant resolves to a canonical `/{type}/{id}` route.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LocationEntryType {
    Site,
    Campus,
    Area,
    JoinedBuilding,
    Building,
    // Mirrors `MSHit::default`; only reachable for index documents predating `type`.
    #[default]
    Room,
    VirtualRoom,
    Poi,
}

/// A single hit from the `entries` index.
///
/// The index mixes geo-entries (sites, buildings, rooms, POIs) with lectures.
/// They share an index but not a shape, so the hit is an enum internally tagged
/// on the `facet` field: serde dispatches each facet value to its variant - the
/// four geo facets reuse [`GeoMSHit`], `lecture` peels off into [`LectureMSHit`].
/// Consumers branch on the variant rather than reaching for fields that only one
/// shape carries.
#[derive(Deserialize, Clone)]
#[serde(tag = "facet", rename_all = "snake_case")]
pub enum MSHit {
    Site(GeoMSHit),
    Building(GeoMSHit),
    Room(GeoMSHit),
    Poi(GeoMSHit),
    Lecture(LectureMSHit),
    Event(EventMSHit),
}

// The `facet` field is consumed as the enum tag, so it is absent here. Fields
// beyond those the merger/formatter read document the meilisearch document
// schema; `#[serde(default)]` keeps them populated and tolerates docs that
// predate a field.
#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct GeoMSHit {
    ms_id: String,
    pub room_code: String,
    pub name: String,
    pub arch_name: Option<String>,
    pub r#type: LocationEntryType,
    pub type_common_name: String,
    pub parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    pub campus: Option<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
}

/// One upcoming occurrence of a lecture, embedded in the lecture document.
///
/// The list lets a client render the expandable lecture row without a second
/// round-trip: `room_name` is the German display name of `room_code` (mirroring
/// the monolingual `name` field of the geo documents), and clicking an
/// occurrence navigates to `/room/<room_code>`.
#[derive(Deserialize, Serialize, Default, Clone, Debug, utoipa::ToSchema)]
pub struct UpcomingEvent {
    /// When the occurrence starts, as an RFC 3339 timestamp.
    #[schema(example = "2024-10-15T08:00:00Z")]
    pub start_at: DateTime<Utc>,
    /// When the occurrence ends, as an RFC 3339 timestamp.
    #[schema(example = "2024-10-15T10:00:00Z")]
    pub end_at: DateTime<Utc>,
    /// The room the occurrence takes place in; navigating to it uses `/room/<room_code>`.
    #[schema(example = "5606.EG.011")]
    pub room_code: String,
    /// The German display name of `room_code`.
    #[schema(example = "Testhörsaal")]
    pub room_name: String,
}

/// A lecture (or tutorial) identity surfaced as the fifth search facet.
///
/// One document per distinct `(title_de, title_en, stp_type)` group, derived
/// from upcoming `calendar` rows by the [`crate::refresh::lectures`] task.
///
/// Every field is required: the refresh task always writes the full shape, so a
/// missing field signals index corruption and should fail the deserialize loudly
/// rather than be papered over with a default.
#[derive(Deserialize, Default, Clone)]
#[expect(
    dead_code,
    reason = "some deserialized fields are not read yet but document the meilisearch document schema"
)]
pub struct LectureMSHit {
    pub ms_id: String,
    /// Mirrors `title_de` for compatibility with the monolingual `name` field.
    pub name: String,
    /// Human label of the `TUMonline` `stp_type` (e.g. "Vorlesung").
    pub type_common_name: String,
    pub title_de: String,
    pub title_en: String,
    pub next_occurrence_at: DateTime<Utc>,
    /// Upcoming occurrences in chronological order; the first element's
    /// `start_at` matches `next_occurrence_at`.
    pub upcoming: Vec<UpcomingEvent>,
    pub parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    rank: i32,
}

/// Coordinates in Meilisearch's `_geo` document shape (note `lng`, not `lon`).
#[derive(Deserialize, Clone, Copy)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

/// A campus event surfaced as the sixth search facet.
///
/// One document per `events.csv` row, exported by the data pipeline into
/// `search_data.json`. Carries everything a client needs to pre-fill the event
/// proposal form, so picking a search hit needs no second round-trip.
///
/// Every field is required, with one exception: the pipeline always writes the
/// full shape, so a missing field signals index corruption and should fail the
/// deserialize loudly rather than be papered over with a default. The two image
/// crop offsets are the exception - they were added after the facet shipped, so
/// documents indexed before then lack them and must default to `0`.
#[derive(Deserialize, Clone)]
pub struct EventMSHit {
    /// The `event_<hash>` identity shared by the CSV row and its key-named images.
    pub ms_id: String,
    pub name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub description: String,
    pub organising_org_id: i32,
    /// The `/cdn/thumb/…` delivery path of the event image.
    pub image: String,
    pub image_author: String,
    /// Crop offset of the thumbnail image: pixels to shift the crop window along the image's longer axis.
    #[serde(default)]
    pub image_thumb_offset: i32,
    /// Crop offset of the header image: pixels to shift the crop window along the image's longer axis.
    #[serde(default)]
    pub image_header_offset: i32,
    #[serde(rename = "_geo")]
    pub coords: GeoPoint,
}

impl Default for MSHit {
    fn default() -> Self {
        Self::Room(GeoMSHit::default())
    }
}

impl MSHit {
    /// The display name, regardless of variant. Every variant carries a `name`
    /// (lectures mirror `title_de` into it), so highlighting works uniformly.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Site(geo) | Self::Building(geo) | Self::Room(geo) | Self::Poi(geo) => &geo.name,
            Self::Lecture(lecture) => &lecture.name,
            Self::Event(event) => &event.name,
        }
    }
}

// Debug intentionally shows only the human-meaningful fields for log readability.
impl Debug for MSHit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Site(geo) | Self::Building(geo) | Self::Room(geo) | Self::Poi(geo) => f
                .debug_struct("MSHit::Geo")
                .field("room_code", &geo.room_code)
                .field("name", &geo.name)
                .finish(),
            Self::Lecture(lecture) => f
                .debug_struct("MSHit::Lecture")
                .field("title_de", &lecture.title_de)
                .finish(),
            Self::Event(event) => f
                .debug_struct("MSHit::Event")
                .field("ms_id", &event.ms_id)
                .field("name", &event.name)
                .finish(),
        }
    }
}

#[derive(Clone)]
pub struct GeoEntryQuery {
    client: Client,
    query: String,
    limits: Limits,
    formatting_config: FormattingConfig,
    user_filter: String,
    sorting: Vec<String>,
}

impl From<(&Client, String, &Limits, &FormattingConfig)> for GeoEntryQuery {
    fn from(
        (client, query, limits, formatting_config): (&Client, String, &Limits, &FormattingConfig),
    ) -> Self {
        Self {
            client: client.clone(),
            query,
            limits: limits.clone(),
            formatting_config: formatting_config.clone(),
            user_filter: String::new(),
            sorting: Vec::new(),
        }
    }
}

impl GeoEntryQuery {
    // add sorting constraints
    pub fn with_sorting(&mut self, sortation: &impl ToString) {
        self.sorting.push(sortation.to_string());
    }
    // add filtering constraints
    pub fn with_filtering(&mut self, ms_filter: &impl ToString) {
        let extra = ms_filter.to_string();
        if !extra.is_empty() {
            if !self.user_filter.is_empty() {
                self.user_filter.push_str(" AND ");
            }
            self.user_filter.push_str(&extra);
        }
    }

    pub async fn execute(self) -> Result<FederatedMultiSearchResponse<MSHit>, Error> {
        let entries = self.client.index(ENTRIES_INDEX);
        let sorting: Vec<&str> = self.sorting.iter().map(String::as_str).collect();
        // Editions of a recurring event share a name and tie on every text
        // ranking rule; sorting the event query by `starts_at` descending
        // surfaces the newest edition first. The pipeline normalises the field
        // to UTC, so Meilisearch's lexicographic string sort is chronological.
        let event_sorting: Vec<&str> = sorting.iter().copied().chain(["starts_at:desc"]).collect();
        // The event facet is default-disabled: a zero cap drops its query from
        // the federation entirely, keeping the request (and thus the result
        // set) identical to one predating the facet.
        let active_facets: Vec<&str> = FACETS
            .iter()
            .copied()
            .filter(|facet| *facet != EVENT_FACET || self.limits.events_count > 0)
            .collect();
        // One filter per facet, ordered to match `active_facets` so callers can
        // reason about per-facet behavior consistently.
        let per_facet_filters: Vec<String> = active_facets
            .iter()
            .map(|f| compose_filter(&facet_eq(f), &self.user_filter))
            .collect();

        // Per-query `limit` is rejected by Meilisearch in federated mode; the
        // global cap lives on `federation.limit` instead. `merge_facets` is
        // set so the response uses the standard `facetDistribution` shape -
        // the SDK's `facets_by_index` field type does not match Meilisearch's
        // per-index keying and would fail to deserialise.
        let mut facets_by_index = HashMap::new();
        facets_by_index.insert(ENTRIES_INDEX.to_string(), vec![FACET_FIELD.to_string()]);

        // `per_facet_filters` is built from `active_facets`, so zipping recovers
        // each filter's facet. The lecture and event queries carry a sub-unit
        // federation weight so their hits are demoted relative to the geo facets
        // when Meilisearch merges the per-facet result sets by weighted
        // `_rankingScore`.
        let mut multi = self.client.multi_search();
        for (facet, filter) in active_facets.iter().zip(&per_facet_filters) {
            match *facet {
                LECTURE_FACET => {
                    let query = self.facet_query(&entries, filter, &sorting);
                    multi.with_search_query_and_weight(query, LECTURE_FEDERATION_WEIGHT);
                }
                EVENT_FACET => {
                    let query = self.facet_query(&entries, filter, &event_sorting);
                    multi.with_search_query_and_weight(query, EVENT_FEDERATION_WEIGHT);
                }
                _ => {
                    let query = self.facet_query(&entries, filter, &sorting);
                    multi.with_search_query(query);
                }
            }
        }
        multi
            .with_federation(FederationOptions {
                limit: Some(
                    self.limits
                        .per_facet_total()
                        .saturating_mul(FEDERATION_OVERFETCH_FACTOR),
                ),
                facets_by_index: Some(facets_by_index),
                merge_facets: Some(MergeFacets::default()),
                ..FederationOptions::default()
            })
            .execute::<MSHit>()
            .await
    }

    fn facet_query<'a>(
        &'a self,
        entries: &'a Index,
        filter: &'a str,
        sorting: &'a [&'a str],
    ) -> SearchQuery<'a, meilisearch_sdk::DefaultHttpClient> {
        SearchQuery::new(entries)
            .with_query(&self.query)
            .with_filter(filter)
            .with_sort(sorting)
            .with_highlight_pre_tag(&self.formatting_config.highlighting.pre)
            .with_highlight_post_tag(&self.formatting_config.highlighting.post)
            .with_attributes_to_highlight(Selectors::Some(&["name"]))
            .with_show_matches_position(true)
            .build()
    }
}

fn facet_eq(value: &str) -> String {
    format!("{FACET_FIELD} = \"{value}\"")
}

fn compose_filter(facet_filter: &str, user_filter: &str) -> String {
    if user_filter.is_empty() {
        facet_filter.to_string()
    } else {
        format!("{facet_filter} AND {user_filter}")
    }
}

#[expect(
    clippy::missing_fields_in_debug,
    reason = "Debug intentionally elides the meilisearch client; only request shape matters in logs"
)]
impl Debug for GeoEntryQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut base = f.debug_struct("GeoEntryQuery");
        base.field("query", &self.query)
            .field("limits", &self.limits)
            .field("highlighting", &self.formatting_config.highlighting);
        if !self.user_filter.is_empty() {
            base.field("user_filter", &self.user_filter);
        }
        if !self.sorting.is_empty() {
            base.field("sorting", &self.sorting);
        }
        base.finish()
    }
}
