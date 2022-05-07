use awc::Client;

use crate::core::search::search_executor::preprocess::SearchToken;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashMap;

/// Input into MeiliSearch
pub(super) struct MSSearchArgs<'a> {
    pub(super) q: &'a str,
    pub(super) filter: Option<MSSearchFilter>,
    pub(super) limit: usize,
}

#[derive(Debug)]
pub(super) struct MSSearchFilter {
    facet: Vec<String>,
}

#[derive(Serialize)]
struct MSQuery<'a> {
    q: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Vec<Vec<String>>>,
    limit: usize,
    #[serde(rename = "facetsDistribution")]
    facets_distribution: Vec<String>,
}

/// Result format of MeiliSearch.
#[derive(Deserialize)]
#[allow(dead_code)]
pub(super) struct MSResults {
    pub(super) hits: Vec<MSHit>,
    #[serde(rename = "nbHits")]
    pub(super) nb_hits: i32,
    #[serde(rename = "facetsDistribution")]
    pub(super) facets_distribution: MSFacetDistribution,
}

#[derive(Deserialize, Debug)]
pub(super) struct MSFacetDistribution {
    pub(super) facet: HashMap<String, i32>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub(super) struct MSHit {
    ms_id: String,
    pub(super) id: String,
    pub(super) name: String,
    pub(super) arch_name: Option<String>,
    pub(super) r#type: String,
    pub(super) type_common_name: String,
    pub(super) parent_building: Vec<String>,
    parent_keywords: Vec<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
}

pub(super) async fn do_meilisearch(client: Client, args: MSSearchArgs<'_>) -> Result<MSResults> {
    let post_data = MSQuery {
        q: args.q,
        filter: match args.filter {
            Some(f) => {
                let mut f_array = Vec::<Vec<String>>::new();
                // Currently only facets, but later also more filters
                let mut sub_f_array = Vec::<String>::new();
                for facet in f.facet {
                    sub_f_array.push(format!("facet = {}", facet)); // TODO: Put in quotes?
                }
                f_array.push(sub_f_array);

                Some(f_array)
            }
            _ => None,
        },
        limit: args.limit,
        facets_distribution: vec!["facet".to_string()],
    };

    let resp_bytes = client
        .post(
            std::env::var("MEILISEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:7700/indexes/entries/search".to_string()),
        )
        .send_json(&post_data)
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    let resp_str = std::str::from_utf8(resp_bytes.as_ref()).unwrap();
    Ok(serde_json::from_str(resp_str)?)
}

pub(super) async fn do_building_search_closed_query(
    client: Client,
    query_string: String,
    limit: usize,
) -> Result<MSResults> {
    do_meilisearch(
        client,
        MSSearchArgs {
            q: &query_string,
            filter: Some(MSSearchFilter {
                facet: vec!["site".to_string(), "building".to_string()],
            }),
            limit,
        },
    )
    .await
}

pub(super) async fn do_room_search(
    client: Client,
    search_tokens: &Vec<SearchToken>,
    limit: usize,
) -> Result<MSResults> {
    let mut q = String::from("");
    for token in search_tokens {
        // It is common that room names are given with four-digits with the first digit
        // being the level. In this case we add splitted terms search well, which could give
        // results if the 4-digit-token doesn't, but still the 4-digit-token should usually
        // take precedence.
        let s = if token.s.len() == 4
            && match token.s.chars().next().unwrap() {
                '0' | '1' | '2' => true,
                _ => false,
            }
            && token.s.chars().all(char::is_numeric)
        {
            format!(
                "{} {} {}",
                token.s,
                token.s.get(0..1).unwrap(),
                token.s.get(1..4).unwrap()
            )
        } else {
            token.s.clone()
        };

        if token.closed && !token.quoted {
            q.push_str(&format!("{} ", s));
        } else {
            q.push_str(&s);
        }
    }

    do_meilisearch(
        client,
        MSSearchArgs {
            q: &q,
            filter: Some(MSSearchFilter {
                facet: vec!["room".to_string()],
            }),
            limit,
        },
    )
    .await
}
