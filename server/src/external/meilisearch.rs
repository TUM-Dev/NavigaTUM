use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use meilisearch_sdk::errors::Error;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::{
    FederatedMultiSearchResponse, FederationOptions, MergeFacets, SearchQuery, Selectors,
};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::routes::search::{FormattingConfig, Limits};

pub(crate) const ENTRIES_INDEX: &str = "entries";
pub(crate) const FACET_FIELD: &str = "facet";
pub(crate) const SITE_FACET: &str = "site";
pub(crate) const BUILDING_FACET: &str = "building";
pub(crate) const ROOM_FACET: &str = "room";
pub(crate) const POI_FACET: &str = "poi";
pub(crate) const LECTURE_FACET: &str = "lecture";
/// Ordered list of facets the search federates over. Order is the priority
/// used by the merger when distributing the over-fetched federation budget
/// across facet caps. Lectures trail the geo facets so a course title only
/// surfaces once the geo entries it competes with have had their say.
pub(crate) const FACETS: &[&str] = &[
    SITE_FACET,
    BUILDING_FACET,
    ROOM_FACET,
    POI_FACET,
    LECTURE_FACET,
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
        }
    }
}

/// Federation has no per-facet quota - it merges by `_rankingScore` only. We
/// over-fetch by this factor so the merger downstream can still fill all
/// per-facet caps when one facet dominates the ranking (observed: queries
/// like `MW1801` returned 14 buildings + 1 room at the natural cap of 15).
/// Empirical; revisit if facet-starvation shows up in metrics.
const FEDERATION_OVERFETCH_FACTOR: usize = 4;

/// A single hit from the `entries` index.
///
/// The index mixes geo-entries (sites, buildings, rooms, POIs) with lectures.
/// They share an index but not a shape, so the hit is modelled as an enum
/// discriminated on the `facet` field: `facet = "lecture"` deserialises into
/// [`LectureMSHit`], everything else into [`GeoMSHit`]. Consumers branch on the
/// variant rather than reaching for fields that only one shape carries.
#[derive(Clone)]
pub enum MSHit {
    Geo(GeoMSHit),
    Lecture(LectureMSHit),
}

#[derive(Default, Clone)]
#[expect(
    dead_code,
    reason = "some deserialized fields are not read yet but document the meilisearch document schema"
)]
pub struct GeoMSHit {
    ms_id: String,
    pub room_code: String,
    pub name: String,
    pub arch_name: Option<String>,
    pub r#type: String,
    pub type_common_name: String,
    pub facet: Option<String>,
    pub parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    pub campus: Option<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
}

/// A lecture (or tutorial) identity surfaced as the fifth search facet.
///
/// One document per distinct `(title_de, title_en, stp_type)` group, derived
/// from upcoming `calendar` rows by the [`crate::refresh::lectures`] task.
#[derive(Default, Clone)]
#[expect(
    dead_code,
    reason = "some deserialized fields are not read yet but document the meilisearch document schema"
)]
pub struct LectureMSHit {
    pub ms_id: String,
    /// Mirrors `title_de` for compatibility with the monolingual `name` field.
    pub name: String,
    pub r#type: String,
    /// Human label of the `TUMonline` `stp_type` (e.g. "Vorlesung").
    pub type_common_name: String,
    pub title_de: String,
    pub title_en: String,
    pub next_occurrence_at: DateTime<Utc>,
    pub parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    rank: i32,
}

impl Default for MSHit {
    fn default() -> Self {
        Self::Geo(GeoMSHit::default())
    }
}

impl MSHit {
    /// The raw `facet` value backing this hit, used by the merger to bucket it.
    #[must_use]
    pub fn facet(&self) -> Option<&str> {
        match self {
            Self::Geo(geo) => geo.facet.as_deref(),
            Self::Lecture(_) => Some(LECTURE_FACET),
        }
    }

    /// The display name, regardless of variant. Both shapes carry a `name`
    /// (lectures mirror `title_de` into it), so highlighting works uniformly.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Geo(geo) => &geo.name,
            Self::Lecture(lecture) => &lecture.name,
        }
    }
}

/// Superset of the fields either variant can carry. Deserialising into one flat
/// struct (rather than `#[serde(tag = ...)]`) lets the four geo facet values all
/// fold into [`GeoMSHit`] while `"lecture"` peels off into [`LectureMSHit`].
#[derive(Deserialize, Default)]
#[serde(default)]
struct MSHitRaw {
    ms_id: String,
    room_code: String,
    name: String,
    arch_name: Option<String>,
    r#type: String,
    type_common_name: String,
    facet: Option<String>,
    parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    campus: Option<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
    title_de: String,
    title_en: String,
    next_occurrence_at: Option<DateTime<Utc>>,
}

impl<'de> Deserialize<'de> for MSHit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = MSHitRaw::deserialize(deserializer)?;
        if raw.facet.as_deref() == Some(LECTURE_FACET) {
            let next_occurrence_at = raw
                .next_occurrence_at
                .ok_or_else(|| D::Error::missing_field("next_occurrence_at"))?;
            Ok(Self::Lecture(LectureMSHit {
                ms_id: raw.ms_id,
                name: raw.name,
                r#type: raw.r#type,
                type_common_name: raw.type_common_name,
                title_de: raw.title_de,
                title_en: raw.title_en,
                next_occurrence_at,
                parent_building_names: raw.parent_building_names,
                parent_keywords: raw.parent_keywords,
                rank: raw.rank,
            }))
        } else {
            Ok(Self::Geo(GeoMSHit {
                ms_id: raw.ms_id,
                room_code: raw.room_code,
                name: raw.name,
                arch_name: raw.arch_name,
                r#type: raw.r#type,
                type_common_name: raw.type_common_name,
                facet: raw.facet,
                parent_building_names: raw.parent_building_names,
                parent_keywords: raw.parent_keywords,
                campus: raw.campus,
                address: raw.address,
                usage: raw.usage,
                rank: raw.rank,
            }))
        }
    }
}

// Debug intentionally shows only the human-meaningful fields for log readability.
impl Debug for MSHit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geo(geo) => f
                .debug_struct("MSHit::Geo")
                .field("room_code", &geo.room_code)
                .field("name", &geo.name)
                .finish(),
            Self::Lecture(lecture) => f
                .debug_struct("MSHit::Lecture")
                .field("title_de", &lecture.title_de)
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
        // One filter per facet, ordered to match `FACETS` so callers can
        // reason about per-facet behavior consistently.
        let per_facet_filters: Vec<String> = FACETS
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

        let mut multi = self.client.multi_search();
        for filter in &per_facet_filters {
            multi.with_search_query(self.facet_query(&entries, filter, &sorting));
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
