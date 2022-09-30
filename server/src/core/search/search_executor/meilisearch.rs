use std::collections::HashMap;

use awc::Client;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use super::preprocess;

// Input into MeiliSearch
pub(super) struct MSSearchArgs {
    pub(super) q: String,
    pub(super) filter: Option<MSSearchFilter>,
    pub(super) limit: usize,
    pub(super) highlighting: (String, String),
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
    facets: Vec<&'a str>,
    #[serde(rename = "highlightPreTag")]
    highlight_pre_tag: &'a str,
    #[serde(rename = "highlightPostTag")]
    highlight_post_tag: &'a str,
    #[serde(rename = "attributesToHighlight")]
    attributes_to_highlight: Vec<&'a str>,
}

/// Result format of MeiliSearch.
#[derive(Deserialize)]
#[allow(dead_code)]
pub(super) struct MSResults {
    pub(super) hits: Vec<MSHit>,
    #[serde(rename = "estimatedTotalHits")]
    pub(super) estimated_total_hits: i32,
    #[serde(rename = "facetDistribution")]
    pub(super) facet_distribution: MSFacetDistribution,
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
    #[serde(rename = "name")]
    pub(super) unformatted_name: String,
    pub(super) arch_name: Option<String>,
    pub(super) r#type: String,
    pub(super) type_common_name: String,
    pub(super) parent_building_names: Vec<String>,
    parent_keywords: Vec<String>,
    pub(super) campus: Option<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
    pub(crate) _formatted: FormattedMSHit,
}

#[derive(Deserialize, Clone)]
pub(super) struct FormattedMSHit {
    // This contains all the atributes of MSHit, but formatted by MS. We only need some, so only some are listed here.
    pub(super) name: String,
}

pub(super) async fn do_meilisearch(client: Client, args: MSSearchArgs) -> Result<MSResults> {
    let post_data = MSQuery {
        q: &args.q,
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
        facets: vec!["facet"],
        highlight_pre_tag: &args.highlighting.0,
        highlight_post_tag: &args.highlighting.1,
        // parsed_id is highlighted by us in postprocessing, because this yields better results
        attributes_to_highlight: vec!["name"],
    };
    // meilisearch should not be a public service as by their docs,
    // this is why we only let users configure the port here :)
    let url = format!(
        "http://{}:{}/indexes/entries/search",
        std::env::var("MIELI_SEARCH_ADDR").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("API_SVC_SERVICE_PORT_MIELI_SEARCH").unwrap_or_else(|_| "7700".to_string())
    );

    // make sure, that meili and the sever are on the same boat when it comes to authentication
    let meili_request = match std::env::var("MEILI_MASTER_KEY") {
        Ok(token) => client.post(url).bearer_auth(token),
        Err(e) => {
            // we can continue with a request here, since it is not a huge security risk
            // if the request goes through our internal network without authentication
            if std::env::var("GIT_COMMIT_SHA").is_ok() {
                // we only warn, if we assume this is production
                warn!("alphanumeric MEILI_MASTER_KEY not found: {:?}", e);
            }
            client.post(url)
        }
    };

    let resp_bytes = meili_request
        .send_json(&post_data)
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    let resp_str = std::str::from_utf8(resp_bytes.as_ref()).unwrap();
    serde_json::from_str(resp_str)
}

pub(super) async fn do_building_search_closed_query(
    client: Client,
    query_string: String,
    limit: usize,
    highlighting: (String, String),
) -> Result<MSResults> {
    do_meilisearch(
        client,
        MSSearchArgs {
            q: query_string,
            filter: Some(MSSearchFilter {
                facet: vec!["site".to_string(), "building".to_string()],
            }),
            limit,
            highlighting,
        },
    )
    .await
}

pub(super) async fn do_room_search(
    client: Client,
    search_tokens: &Vec<preprocess::SearchToken>,
    limit: usize,
    highlighting: (String, String),
) -> Result<MSResults> {
    let mut q = String::from("");
    for token in search_tokens {
        // It is common that room names are given with four-digits with the first digit
        // being the level. In this case we add splitted terms search well, which could give
        // results if the 4-digit-token doesn't, but still the 4-digit-token should usually
        // take precedence.
        let s = if token.s.len() == 4
            && matches!(token.s.chars().next().unwrap(), '0' | '1' | '2')
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
            q,
            filter: Some(MSSearchFilter {
                facet: vec!["room".to_string()],
            }),
            limit,
            highlighting,
        },
    )
    .await
}
