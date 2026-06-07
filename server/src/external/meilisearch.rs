use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

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
/// Ordered list of facets the search federates over. Order is the priority
/// used by the merger when distributing the over-fetched federation budget
/// across facet caps.
pub(crate) const FACETS: &[&str] = &[SITE_FACET, BUILDING_FACET, ROOM_FACET, POI_FACET];

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
}

impl FacetFilter {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Site => SITE_FACET,
            Self::Building => BUILDING_FACET,
            Self::Room => ROOM_FACET,
            Self::Poi => POI_FACET,
        }
    }
}

/// Federation has no per-facet quota - it merges by `_rankingScore` only. We
/// over-fetch by this factor so the merger downstream can still fill all
/// per-facet caps when one facet dominates the ranking (observed: queries
/// like `MW1801` returned 14 buildings + 1 room at the natural cap of 15).
/// Empirical; revisit if facet-starvation shows up in metrics.
const FEDERATION_OVERFETCH_FACTOR: usize = 4;

#[derive(Deserialize, Default, Clone)]
#[expect(
    dead_code,
    reason = "some deserialized fields are not read yet but document the meilisearch document schema"
)]
pub struct MSHit {
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
#[expect(
    clippy::missing_fields_in_debug,
    reason = "Debug intentionally shows only the human-meaningful fields for log readability"
)]
impl Debug for MSHit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MSHit")
            .field("room_code", &self.room_code)
            .field("name", &self.name)
            .finish()
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
