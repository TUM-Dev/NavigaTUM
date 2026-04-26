use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use meilisearch_sdk::client::Client;
use meilisearch_sdk::errors::Error;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::{
    FederatedMultiSearchResponse, FederationOptions, MergeFacets, SearchQuery, Selectors,
};
use serde::Deserialize;

use crate::routes::search::{FormattingConfig, Limits};

pub(crate) const ENTRIES_INDEX: &str = "entries";
pub(crate) const FACET_FIELD: &str = "facet";
pub(crate) const ROOM_FACET: &str = "room";
pub(crate) const BUILDING_FACET: &str = "building";

/// Federation has no per-facet quota — it merges by `_rankingScore` only. We
/// over-fetch by this factor so the merger downstream can still fill both
/// per-facet caps when one facet dominates the ranking (observed: queries
/// like `MW1801` returned 14 buildings + 1 room at the natural cap of 15).
/// Empirical; revisit if facet-starvation shows up in metrics.
const FEDERATION_OVERFETCH_FACTOR: usize = 4;

#[derive(Deserialize, Default, Clone)]
#[allow(dead_code)]
pub struct MSHit {
    ms_id: String,
    pub room_code: String,
    pub name: String,
    pub arch_name: Option<String>,
    pub r#type: String,
    pub type_common_name: String,
    pub parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    pub campus: Option<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
}
impl Debug for MSHit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    pub fn with_sorting(&mut self, sortation: impl ToString) -> Self {
        self.sorting.push(sortation.to_string());
        self.clone()
    }
    // add filtering constraints
    pub fn with_filtering(&mut self, ms_filter: impl ToString) -> Self {
        let extra = ms_filter.to_string();
        if !extra.is_empty() {
            if !self.user_filter.is_empty() {
                self.user_filter.push_str(" AND ");
            }
            self.user_filter.push_str(&extra);
        }
        self.clone()
    }

    pub async fn execute(self) -> Result<FederatedMultiSearchResponse<MSHit>, Error> {
        let entries = self.client.index(ENTRIES_INDEX);
        let sorting: Vec<&str> = self.sorting.iter().map(String::as_str).collect();
        let rooms_filter = compose_filter(&facet_eq(ROOM_FACET), &self.user_filter);
        let buildings_filter = compose_filter(&facet_eq(BUILDING_FACET), &self.user_filter);

        // Per-query `limit` is rejected by Meilisearch in federated mode; the
        // global cap lives on `federation.limit` instead. `merge_facets` is
        // set so the response uses the standard `facetDistribution` shape —
        // the SDK's `facets_by_index` field type does not match Meilisearch's
        // per-index keying and would fail to deserialise.
        let mut facets_by_index = HashMap::new();
        facets_by_index.insert(ENTRIES_INDEX.to_string(), vec![FACET_FIELD.to_string()]);

        let mut multi = self.client.multi_search();
        multi.with_search_query(self.facet_query(&entries, &rooms_filter, &sorting));
        multi.with_search_query(self.facet_query(&entries, &buildings_filter, &sorting));
        multi
            .with_federation(FederationOptions {
                limit: Some(
                    (self.limits.buildings_count + self.limits.rooms_count)
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

impl Debug for GeoEntryQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
