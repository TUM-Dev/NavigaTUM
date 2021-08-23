use std::collections::HashMap;
use std::time::Instant;

use actix_web::client::{Client, ClientBuilder, Connector};
use serde::{Deserialize, Serialize};
use serde_json::Result;

/// Returned search results by this
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

/// Result format of MeiliSearch.
#[derive(Deserialize)]
#[allow(dead_code)]
struct MSResults {
    hits: Vec<MSHit>,
    #[serde(rename = "nbHits")]
    nb_hits: i32,
    #[serde(rename = "processingTimeMs")]
    processing_time_ms: i32,
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

    let results: Vec<ResultEntry> = ms_results
        .hits
        .iter()
        .map(|r| ResultEntry {
            id: r.id.to_string(),
            r#type: r.r#type.to_string(),
            name: r.name.to_string(),
            subtext: format!("{:?}, {}", r.arch_name, r.type_common_name),
        })
        .collect();

    let time_ms = start_time.elapsed().as_millis();
    Ok(SearchResults {
        results,
        nb_hits: ms_results.nb_hits,
        time_ms,
    })
}

async fn do_meilisearch(q: &str, client: &Client) -> Result<MSResults> {
    let mut post_data = HashMap::new();
    post_data.insert("q", q);

    let resp_bytes = client
        .post(
            std::env::var("MEILISEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:7700/indexes/obj/search".to_string()),
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
