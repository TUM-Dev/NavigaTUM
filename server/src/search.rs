use std::collections::HashMap;
use std::time::Instant;

use actix_web::client::{Client, ClientBuilder, Connector};
use serde::{Deserialize, Serialize};
use serde_json::Result;

// Returned search results by this
#[derive(Serialize, Debug)]
pub struct SearchResults {
    results: Vec<ResultEntry>,
    nb_hits: i32,
    time_ms: u128,
}

#[derive(Serialize, Debug)]
pub struct ResultEntry {
    id: String,
    r#type: String,
    name: String,
    subtext: String,
}

// Result format of MeiliSearch.
#[derive(Deserialize)]
#[allow(dead_code, non_snake_case)]
struct MSResults {
    hits: Vec<MSHit>,
    nbHits: i32,
    processingTimeMs: i32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MSHit {
    id: String,
    name: String,
    arch_name: Option<String>,
    r#type: String,
    type_common_name: String,
    parent_building: Vec<String>,
    parent_keywords: Vec<String>,
    address: Option<String>,
    usage: Option<String>,
    rank: i32,
}

pub async fn do_search(q: String) -> Result<SearchResults> {
    let start_time = Instant::now();

    let client = ClientBuilder::new()
        .connector(Connector::new().finish())
        .finish();

    let ms_results = do_meilisearch(&q, &client).await?;

    let mut results = Vec::<ResultEntry>::new();
    for r in ms_results.hits {
        results.push(ResultEntry {
            id: r.id,
            r#type: r.r#type,
            name: r.name,
            subtext: format!("{:?}, {}", r.arch_name, r.type_common_name),
        })
    }

    let time_ms = start_time.elapsed().as_millis();

    let results = SearchResults {
        results: results,
        nb_hits: ms_results.nbHits,
        time_ms: time_ms,
    };

    Ok(results)
}

async fn do_meilisearch(q: &String, client: &Client) -> Result<MSResults> {
    let mut post_data = HashMap::new();
    post_data.insert("q", q);

    let resp_bytes = client
        .post("http://localhost:7700/indexes/obj/search")
        .send_json(&post_data)
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    let resp_str = std::str::from_utf8(resp_bytes.as_ref()).unwrap();
    Ok(serde_json::from_str(resp_str)?)
}
