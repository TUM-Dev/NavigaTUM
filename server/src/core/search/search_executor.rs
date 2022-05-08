use awc::{ClientBuilder, Connector};
use futures::try_join;
use serde::Serialize;

use super::search_executor::preprocess::SearchToken;
use super::search_handler::SanitisedSearchQueryArgs;
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
    let parsed_input = preprocess::parse_input_query(q);

    // Determine what to search for

    // Currently ranking is designed to put buildings at the top if they equally
    // match the term compared to a room. For this reason there is only a search
    // for all entries and only rooms, search matching (and relevant) buildings can be
    // expected to be at the top of the merged search. However sometimes a lot of
    // buildings will be hidden (e.g. building parts), so the extra room search ....
    let client = ClientBuilder::new().connector(Connector::new()).finish();

    let q_default = parsed_input.clone().to_query_string();

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
        Ok((res_merged, res_buildings, res_rooms)) => {
            merge_search_results(args, &search_tokens, res_merged, res_buildings, res_rooms)
        }
        Err(e) => {
            // error should be serde_json::error
            error!("Error searching for results: {:?}", e);
            vec![]
        }
    };
}

fn merge_search_results(
    args: SanitisedSearchQueryArgs,
    search_tokens: &Vec<SearchToken>,
    res_merged: meilisearch::MSResults,
    res_buildings: meilisearch::MSResults,
    res_rooms: meilisearch::MSResults,
) -> Vec<SearchResultsSection> {
    // First look up which buildings did match even with a closed query.
    // We can consider them more relevant.
    let mut closed_matching_buildings = Vec::<String>::new();
    for hit in res_buildings.hits {
        closed_matching_buildings.push(hit.id);
    }

    let facet = res_merged.facets_distribution.facet;
    let mut section_buildings = SearchResultsSection {
        facet: "sites_buildings".to_string(),
        entries: Vec::<ResultEntry>::new(),
        n_visible: None,
        nb_hits: facet.get("site").unwrap_or_else(|| &0)
            + facet.get("building").unwrap_or_else(|| &0),
    };
    let mut section_rooms = SearchResultsSection {
        facet: "rooms".to_string(),
        entries: Vec::<ResultEntry>::new(),
        n_visible: None,
        nb_hits: res_rooms.nb_hits,
    };

    // TODO: Collapse joined buildings
    // let mut observed_joined_buildings = Vec::<String>::new();
    let mut observed_ids = Vec::<String>::new();
    for mut hit in [res_merged.hits, res_rooms.hits].concat() {
        if observed_ids.contains(&hit.id) {
            continue;
        }; // No duplicates

        // Total limit reached (does only count visible results)
        if section_rooms.entries.len()
            + section_buildings
                .n_visible
                .unwrap_or_else(|| section_buildings.entries.len())
            >= args.limit_all
        {
            break;
        }

        // Find out where it matches TODO: Improve
        let highlighted_name = postprocess::highlight_matches(&hit.name, &search_tokens);
        let highlighted_arch_name = match &hit.arch_name {
            Some(arch_name) => postprocess::highlight_matches(arch_name, &search_tokens),
            None => String::from(""),
        };

        match hit.r#type.as_str() {
            "campus" | "site" | "area" | "building" | "joined_building" => {
                if section_buildings.entries.len() < args.limit_buildings {
                    push_to_room_queue(&mut section_buildings, &mut hit, highlighted_name);
                }
            }
            "room" | "virtual_room" => {
                if section_rooms.entries.len() < args.limit_rooms {
                    push_to_sections_queue(
                        &mut section_rooms,
                        &mut hit,
                        &search_tokens,
                        highlighted_name,
                        highlighted_arch_name,
                    );

                    // The first room in the results 'freezes' the number of visible buildings
                    if section_buildings.n_visible.is_none() && section_rooms.entries.len() == 1 {
                        section_buildings.n_visible = Some(section_buildings.entries.len());
                    }
                }
            }
            _ => {}
        };

        observed_ids.push(hit.id);
    }

    match section_buildings.n_visible {
        Some(0) => vec![section_rooms, section_buildings],
        _ => vec![section_buildings, section_rooms],
    }
}

fn push_to_room_queue(
    section_buildings: &mut SearchResultsSection,
    hit: &mut meilisearch::MSHit,
    highlighted_name: String,
) {
    section_buildings.entries.push(ResultEntry {
        id: hit.id.to_string(),
        r#type: hit.r#type.to_string(),
        name: highlighted_name,
        subtext: format!("{}", hit.type_common_name),
        subtext_bold: None,
        parsed_id: None,
    });
}

fn push_to_sections_queue(
    section_rooms: &mut SearchResultsSection,
    hit: &mut meilisearch::MSHit,
    search_tokens: &&Vec<SearchToken>,
    highlighted_name: String,
    highlighted_arch_name: String,
) {
    // Test whether the query matches some common room id formats
    let parsed_id = postprocess::parse_room_formats(&search_tokens, &hit);

    section_rooms.entries.push(ResultEntry {
        id: hit.id.to_string(),
        r#type: hit.r#type.to_string(),
        name: highlighted_name,
        subtext: format!(
            "{}",
            if hit.parent_building.len() > 0 {
                &hit.parent_building[0]
            } else {
                ""
            }
        ),
        subtext_bold: if parsed_id.is_some() {
            Some(hit.arch_name.clone().unwrap_or_default())
        } else {
            Some(highlighted_arch_name)
        },
        parsed_id,
    });
}
