use crate::core::search::search_executor::preprocess;
use crate::core::search::SanitisedSearchQueryArgs;
use meilisearch_sdk::errors::Error;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::{MultiSearchResponse, SearchQuery, Selectors};
use meilisearch_sdk::Client;
use serde::Deserialize;

fn ms_url() -> String {
    // meilisearch should not be a public service as by their docs
    format!(
        "http://{}:{}/",
        std::env::var("MIELISEARCH_HOST").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("MIELISEARCH_PORT").unwrap_or_else(|_| "7700".to_string())
    )
}

#[derive(Deserialize, Clone, Debug)]
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

pub(super) struct GeoEntryQuery {
    parsed_input: preprocess::SearchInput,
    args: SanitisedSearchQueryArgs,
    highlighting: (String, String),
}

impl GeoEntryQuery {
    pub fn from(
        parsed_input: &preprocess::SearchInput,
        args: &SanitisedSearchQueryArgs,
        highlighting: &(String, String),
    ) -> Self {
        Self {
            parsed_input: parsed_input.clone(),
            args: *args,
            highlighting: highlighting.clone(),
        }
    }
    pub async fn execute(self) -> Result<MultiSearchResponse<MSHit>, Error> {
        let q_default = self.parsed_input.to_default_query();
        let client = Client::new(ms_url(), std::env::var("MEILI_MASTER_KEY").ok());
        let entries = client.index("entries");

        // Currently ranking is designed to put buildings at the top if they equally
        // match the term compared to a room. For this reason there is only a search
        // for all entries and only rooms, search matching (and relevant) buildings can be
        // expected to be at the top of the merged search. However sometimes a lot of
        // buildings will be hidden (e.g. building parts), so the extra room search ....
        client
            .multi_search()
            .with_search_query(self.merged_query(&entries, &q_default))
            .with_search_query(self.buildings_query(&entries, &q_default))
            .with_search_query(self.rooms_query(&entries, &self.parsed_input.to_room_query()))
            .execute::<MSHit>()
            .await
    }

    fn merged_query<'a>(&'a self, entries: &'a Index, query: &'a str) -> SearchQuery<'a> {
        SearchQuery::new(entries)
            .with_facets(Selectors::Some(&["facet"]))
            .with_highlight_pre_tag(&self.highlighting.0)
            .with_highlight_post_tag(&self.highlighting.1)
            .with_attributes_to_highlight(Selectors::Some(&["name"]))
            .with_query(query)
            .with_limit(self.args.limit_all)
            .build()
    }

    fn buildings_query<'a>(&'a self, entries: &'a Index, query: &'a str) -> SearchQuery<'a> {
        SearchQuery::new(entries)
            .with_facets(Selectors::Some(&["facet"]))
            .with_highlight_pre_tag(&self.highlighting.0)
            .with_highlight_post_tag(&self.highlighting.1)
            .with_attributes_to_highlight(Selectors::Some(&["name"]))
            .with_query(query)
            .with_limit(2 * self.args.limit_buildings) // we might do reordering later
            .with_filter("facet = \"building\"")
            .build()
    }

    fn rooms_query<'a>(&'a self, entries: &'a Index, query: &'a str) -> SearchQuery<'a> {
        SearchQuery::new(entries)
            .with_facets(Selectors::Some(&["facet"]))
            .with_highlight_pre_tag(&self.highlighting.0)
            .with_highlight_post_tag(&self.highlighting.1)
            .with_attributes_to_highlight(Selectors::Some(&["name"]))
            .with_query(query)
            .with_limit(self.args.limit_rooms)
            .with_filter("facet = \"room\"")
            .build()
    }
}
