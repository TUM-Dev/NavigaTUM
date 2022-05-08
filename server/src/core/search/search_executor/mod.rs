use awc::{ClientBuilder, Connector};
use futures::try_join;
use serde::Serialize;

use super::SanitisedSearchQueryArgs;
use cached::proc_macro::cached;
use log::error;

mod meilisearch;
mod postprocess;
mod preprocess;

#[derive(Serialize, Debug, Clone)]
pub struct SearchResultsSection {
    facet: String,
    entries: Vec<ResultEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_visible: Option<usize>,
    nb_hits: i32,
}

#[derive(Serialize, Debug, Clone)]
struct ResultEntry {
    id: String,
    r#type: String,
    name: String,
    subtext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtext_bold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parsed_id: Option<String>,
}

// size=100 seems to be about 10M
#[cached(size = 500)]
pub async fn do_geoentry_search(
    q: String,
    args: SanitisedSearchQueryArgs,
) -> Vec<SearchResultsSection> {
    let parsed_input = preprocess::parse_input_query(q.as_str());

    // Determine what to search for

    // Currently ranking is designed to put buildings at the top if they equally
    // match the term compared to a room. For this reason there is only a search
    // for all entries and only rooms, search matching (and relevant) buildings can be
    // expected to be at the top of the merged search. However sometimes a lot of
    // buildings will be hidden (e.g. building parts), so the extra room search ....
    let client = ClientBuilder::new().connector(Connector::new()).finish();

    let q_default = parsed_input.to_query_string();

    let fut_res_merged = meilisearch::do_meilisearch(
        client.clone(),
        meilisearch::MSSearchArgs {
            q: &q_default,
            filter: None,
            limit: args.limit_all,
        },
    );
    let search_tokens = parsed_input.tokens;
    // Building limit multiplied by two because we might do reordering later
    let fut_res_buildings = meilisearch::do_building_search_closed_query(
        client.clone(),
        q_default.clone(),
        2 * args.limit_buildings,
    );
    let fut_res_rooms =
        meilisearch::do_room_search(client.clone(), &search_tokens, args.limit_rooms);

    return match try_join!(fut_res_merged, fut_res_buildings, fut_res_rooms) {
        Ok((res_merged, res_buildings, res_rooms)) => postprocess::merge_search_results(
            &args,
            &search_tokens,
            res_merged,
            res_buildings,
            res_rooms,
        ),
        Err(e) => {
            // error should be serde_json::error
            error!("Error searching for results: {:?}", e);
            vec![]
        }
    };
}
