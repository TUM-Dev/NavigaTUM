use meilisearch_sdk::client::Client;
use meilisearch_sdk::errors::Error;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::{MultiSearchResponse, SearchQuery, Selectors};
use serde::Deserialize;

use crate::search::search_executor::parser::{Filter, ParsedQuery, TextToken};
use crate::search::{Highlighting, Limits};

#[derive(Deserialize, Default, Clone, Debug)]
#[allow(dead_code)]
pub(super) struct MSHit {
    ms_id: String,
    pub(super) id: String,
    pub(super) name: String,
    pub(super) arch_name: Option<String>,
    pub(super) r#type: String,
    pub(super) type_common_name: String,
    pub(super) parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    pub(super) campus: Option<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
}

struct GeoEntryFilters {
    default: String,
    rooms: String,
    buildings: String,
}
impl GeoEntryFilters {
    fn from(filters: &Filter) -> Self {
        let ms_filter = filters.as_meilisearch_filters();
        let separator = if ms_filter.is_empty() { " " } else { " AND " };
        Self {
            default: ms_filter.clone(),
            buildings: format!("facet = \"building\"{separator}{ms_filter}"),
            rooms: format!("facet = \"room\"{separator}{ms_filter}"),
        }
    }
}

pub(super) struct GeoEntryQuery {
    parsed_input: ParsedQuery,
    args: Limits,
    highlighting: Highlighting,
    filters: GeoEntryFilters,
    sorting: Vec<String>,
}

impl GeoEntryQuery {
    pub fn from(parsed_input: &ParsedQuery, args: &Limits, highlighting: &Highlighting) -> Self {
        Self {
            parsed_input: parsed_input.clone(),
            args: *args,
            highlighting: highlighting.clone(),
            filters: GeoEntryFilters::from(&parsed_input.filters),
            sorting: parsed_input.sorting.as_meilisearch_sorting(),
        }
    }
    pub async fn execute(self) -> Result<MultiSearchResponse<MSHit>, Error> {
        let q_default = self.prompt_for_querying();
        let ms_url =
            std::env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
        let client = Client::new(ms_url, std::env::var("MEILI_MASTER_KEY").ok())?;
        let entries = client.index("entries");

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
        client
            .multi_search()
            .with_search_query(
                self.merged_query(&entries, &q_default)
                    .with_sort(&sorting)
                    .build(),
            )
            .with_search_query(
                self.buildings_query(&entries, &q_default)
                    .with_sort(&sorting)
                    .build(),
            )
            .with_search_query(
                self.rooms_query(&entries, &self.prompt_for_querying_room())
                    .with_sort(&sorting)
                    .build(),
            )
            .execute::<MSHit>()
            .await
    }

    fn prompt_for_querying(&self) -> String {
        self.parsed_input
            .tokens
            .clone()
            .into_iter()
            .map(|s| match s {
                TextToken::Text(t) => t,
                TextToken::SplittableText((t1, t2)) => format!("{t1}{t2}"),
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
    fn prompt_for_querying_room(&self) -> String {
        self.parsed_input
            .tokens
            .clone()
            .into_iter()
            .map(|s| match s {
                TextToken::Text(t) => t,
                TextToken::SplittableText((t1, t2)) => format!("{t1} {t2} {t1}{t2}"),
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn common_query<'b: 'a, 'a>(
        &'b self,
        entries: &'a Index,
    ) -> SearchQuery<'a, meilisearch_sdk::DefaultHttpClient> {
        SearchQuery::new(entries)
            .with_facets(Selectors::Some(&["facet"]))
            .with_highlight_pre_tag(&self.highlighting.pre)
            .with_highlight_post_tag(&self.highlighting.post)
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
            .with_limit(self.args.total_count)
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
            .with_limit(2 * self.args.buildings_count) // we might do reordering later
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
            .with_limit(self.args.rooms_count)
            .with_filter(&self.filters.rooms)
            .build()
    }
}
