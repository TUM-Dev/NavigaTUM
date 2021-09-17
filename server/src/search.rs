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

    // MeiliSearch does automatically split tokens, but for the kind of tokens
    // in the entry index, this is unreliable. For this reason, we split the tokens
    // at common positions and forward the split queries to MeiliSearch, which does
    // also auto-merge them up to three word. We can later account for this by moving
    // correctly merged tokens back up in results.
    // Note: This process seems to account for +0.2ms request time on average because
    //       it increases the number of tokens.
    let tokens_by_space = q.split_whitespace();
    let mut q_splitted_vec = Vec::<String>::new();
    for token in tokens_by_space {
        // Tokens like "AB123" (alpha and then numeric) are split int "AB 123".
        // Currently, a maximum of one split is done here.
        let mut has_done_split = false;
        if token.len() > 1 {
            let mut chars = token.chars();
            if chars.next().unwrap().is_alphabetic() {
                let mut alphabetic_part = Vec::<char>::new();
                while let Some(c) = chars.next() {
                    if c.is_alphabetic() {
                        alphabetic_part.push(c);
                    } else if c.is_numeric() {
                        q_splitted_vec.push(alphabetic_part.iter().collect::<String>());
                        let mut rest_part = Vec::<char>::new();
                        rest_part.push(c);
                        rest_part.extend(chars);
                        q_splitted_vec.push(rest_part.iter().collect::<String>());
                        has_done_split = true;
                        break;
                    } else { // Doesn't match pattern, ignore this token then
                        break;
                    }
                }
            }
        }

        // Numeric only strings
        // It is common that room names are given with four-digist with the first digit
        // being the level. So in this special case we do a split at this point, however
        // only if the query has less than three tokens (TODO), so MeiliSearch still auto-merges.
        // This is important because then we can assume it is still possible to reorder
        // the results and put exact merged results back up.
        if !has_done_split &&
           token.len() == 4 &&
           token.chars().all(char::is_numeric) {
            let mut chars = token.chars();
            let first_char = chars.next().unwrap();
            if first_char == '0' || first_char == '1' || first_char == '2' {
                q_splitted_vec.push(first_char.to_string());
                q_splitted_vec.push(chars.as_str().to_string());
                has_done_split = true;
            }
        }

        if !has_done_split {
            q_splitted_vec.push(token.to_string());
        }
    }
    let q_splitted = q_splitted_vec.join(" ");

    let client = ClientBuilder::new()
        .connector(Connector::new().finish())
        .finish();

    let ms_results = do_meilisearch(&q_splitted, client).await?;

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

async fn do_meilisearch(q: &str, client: Client) -> Result<MSResults> {
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
