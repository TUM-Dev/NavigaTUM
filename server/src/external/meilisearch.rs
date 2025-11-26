use meilisearch_sdk::client::Client;
use meilisearch_sdk::errors::Error;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::{MultiSearchResponse, SearchQuery, Selectors};
use serde::Deserialize;
use std::fmt::{Debug, Formatter};

use crate::routes::search::{FormattingConfig, Limits};

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
struct GeoEntryFilters {
    default: String,
    rooms: String,
    buildings: String,
}
impl Default for GeoEntryFilters {
    fn default() -> Self {
        Self {
            default: "".to_string(),
            rooms: "facet = \"room\"".to_string(),
            buildings: "facet = \"building\"".to_string(),
        }
    }
}
impl Debug for GeoEntryFilters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_struct("GeoEntryFilters");
        if !self.default.is_empty() {
            base.field("default", &self.default);
        }
        if !self.rooms.is_empty() {
            base.field("rooms", &self.rooms);
        }
        if !self.buildings.is_empty() {
            base.field("buildings", &self.buildings);
        }
        base.finish()
    }
}

impl From<&String> for GeoEntryFilters {
    fn from(ms_filter: &String) -> Self {
        Self::default().with_filter(ms_filter)
    }
}

impl GeoEntryFilters {
    pub fn with_filter(&mut self, ms_filter: impl ToString) -> Self {
        let ms_filter = ms_filter.to_string();
        // --
        if self.default.is_empty() && !ms_filter.is_empty() {
            self.default.push_str(" AND ")
        }
        self.default.push_str(&ms_filter);
        // --
        if self.buildings.is_empty() && !ms_filter.is_empty() {
            self.buildings.push_str(" AND ")
        }
        self.buildings.push_str(&ms_filter);
        // --
        if self.rooms.is_empty() && !ms_filter.is_empty() {
            self.rooms.push_str(" AND ")
        }
        self.rooms.push_str(&ms_filter);
        self.clone()
    }
}

#[derive(Clone)]
pub struct GeoEntryQuery {
    client: Client,
    query: String,
    limits: Limits,
    formatting_config: FormattingConfig,
    filters: GeoEntryFilters,
    sorting: Vec<String>,
}

impl From<(&Client, String, &Limits, &FormattingConfig)> for GeoEntryQuery {
    fn from(
        (client, query, limits, formatting_config): (&Client, String, &Limits, &FormattingConfig),
    ) -> Self {
        Self {
            client: client.clone(),
            query,
            limits: *limits,
            formatting_config: formatting_config.clone(),
            filters: GeoEntryFilters::default(),
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
        self.filters.with_filter(ms_filter);
        self.clone()
    }
    pub async fn execute(self) -> Result<MultiSearchResponse<MSHit>, Error> {
        let entries = self.client.index("entries");

        // due to lifetime shenanigans this is added here (I can't make it move down to the other statements)
        // If you can make it, please propose a PR, I know that this is really hacky ^^
        let sorting = self
            .sorting
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>();

        // Currently ranking is designed to put buildings at the top if they equally
        // match the term compared to a room. For this reason there is only a search
        // for all entries and only rooms, search matching (and relevant) buildings can be
        // expected to be at the top of the merged search. However sometimes a lot of
        // buildings will be hidden (e.g. building parts), so the extra room search ....
        self.client
            .multi_search()
            .with_search_query(
                self.merged_query(&entries, &self.query)
                    .with_sort(&sorting)
                    .build(),
            )
            .with_search_query(
                self.buildings_query(&entries, &self.query)
                    .with_sort(&sorting)
                    .build(),
            )
            .with_search_query(
                self.rooms_query(&entries, &self.query)
                    .with_sort(&sorting)
                    .build(),
            )
            .execute::<MSHit>()
            .await
    }

    fn common_query<'b: 'a, 'a>(
        &'b self,
        entries: &'a Index,
    ) -> SearchQuery<'a, meilisearch_sdk::DefaultHttpClient> {
        SearchQuery::new(entries)
            .with_facets(Selectors::Some(&["facet"]))
            .with_highlight_pre_tag(&self.formatting_config.highlighting.pre)
            .with_highlight_post_tag(&self.formatting_config.highlighting.post)
            .with_attributes_to_highlight(Selectors::Some(&["name"]))
            .build()
    }

    fn merged_query<'a>(
        &'a self,
        entries: &'a Index,
        query: &'a str,
    ) -> SearchQuery<'a, meilisearch_sdk::DefaultHttpClient> {
        let mut s = self
            .common_query(entries)
            .with_query(query)
            .with_limit(self.limits.total_count)
            .build();
        if !self.filters.default.is_empty() {
            s = s.with_filter(&self.filters.default).build();
        }
        s
    }

    fn buildings_query<'a>(
        &'a self,
        entries: &'a Index,
        query: &'a str,
    ) -> SearchQuery<'a, meilisearch_sdk::DefaultHttpClient> {
        self.common_query(entries)
            .with_query(query)
            .with_limit(2 * self.limits.buildings_count) // we might do reordering later
            .with_filter(&self.filters.buildings)
            .build()
    }

    fn rooms_query<'a>(
        &'a self,
        entries: &'a Index,
        query: &'a str,
    ) -> SearchQuery<'a, meilisearch_sdk::DefaultHttpClient> {
        self.common_query(entries)
            .with_query(query)
            .with_limit(self.limits.rooms_count)
            .with_filter(&self.filters.rooms)
            .build()
    }
}

impl Debug for GeoEntryQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_struct("GeoEntryQuery");
        base.field("query", &self.query)
            .field("limits", &self.limits)
            .field("highlighting", &self.formatting_config.highlighting)
            .field("filters", &self.filters);
        if !self.sorting.is_empty() {
            base.field("sorting", &self.sorting);
        }
        base.finish()
    }
}
