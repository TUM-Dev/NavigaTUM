use std::collections::HashMap;
use std::time::Instant;

use actix_web::client::{Client, ClientBuilder, Connector};
use serde::{Deserialize, Serialize};
use serde_json::Result;


#[derive(Debug)]
struct InputToken {
    s: String,
    regular_split: bool,
    closed: bool,
}

#[derive(Debug)]
struct SearchToken {
    s: String,
    regular_split: bool,
    closed: bool,
    quoted: bool,
}

#[derive(Debug)]
struct SearchFilter {
    parent: Option<Vec::<String>>,
    r#type: Option<Vec::<String>>,
    usage: Option<Vec::<String>>,
}

#[derive(Debug)]
struct SearchInput {
    tokens: Vec::<SearchToken>,
    filter: SearchFilter,
}

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

    let parsed = parse_input_query(&q);
    let new_input_query = build_query_string(parsed.tokens);

    // MeiliSearch does automatically split tokens, but for the kind of tokens
    // in the entry index, this is unreliable. For this reason, we split the tokens
    // at common positions and forward the split queries to MeiliSearch, which does
    // also auto-merge them up to three word. We can later account for this by moving
    // correctly merged tokens back up in results.
    // Note: This process seems to account for +0.2ms request time on average because
    //       it increases the number of tokens.
    let tokens_by_space = new_input_query.split_whitespace();
    let mut q_splitted_vec = Vec::<String>::new();
    for token in tokens_by_space {
        // Tokens like "AB123" (alpha and then numeric) are split int "AB 123".
        // Currently, a maximum of one split is done here.
        let mut has_done_split = false;

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

fn build_query_string(search_tokens: Vec::<SearchToken>) -> String {
    //let s_parts = Vec::<String>::new();
    let mut s = String::from("");
    for token in search_tokens {
        if token.closed && !token.quoted {
            s.push_str(&format!("{} ", token.s));
        } else {
            s.push_str(&token.s.clone());
        }
    }

    s
}

fn parse_input_query(q: &str) -> SearchInput {
    let input_tokens = tokenize_input_query(&q);

    let mut search_tokens = Vec::<SearchToken>::new();
    let mut search_filter = SearchFilter {
        parent: None,
        r#type: None,
        usage: None,
    };
    for token in input_tokens {
        // Quoted tokens are ignored. Note this also marks unclosed tokens at the end as quoted.
        if token.s.starts_with("\"") {
            search_tokens.push(SearchToken {
                s: token.s,
                regular_split: token.regular_split,
                closed: token.closed,
                quoted: true,
            });
        } else {
            // Parse filters
            let mut is_filter = false;
            for prefix in vec!["in:", "@", "usage:", "nutzung:", "=", "type:"] {
                if (&token.s).starts_with(prefix) {
                    is_filter = true;

                    let v = token.s.trim_start_matches(prefix)
                                   .trim_start_matches("\"")
                                   .trim_end_matches("\"");
                    if v.len() == 0 { continue }; // e.g. ' in: ', ' @ ', ' in:"" ' are ignored, TODO: autosuggest

                    let filter = match prefix {
                        "in:" | "@" => Some(&mut search_filter.parent),
                        "usage:" | "nutzung:" | "=" => Some(&mut search_filter.usage),
                        "type:" => Some(&mut search_filter.r#type),
                        _ => None,
                    };

                    if let Some(Some(f)) = filter {
                        f.push(v.to_string());
                    } else {
                        *filter.unwrap() = Some(vec![v.to_string()]);
                    }

                    break;
                }
            }

            if !is_filter {
                search_tokens.push(SearchToken {
                    s: token.s,
                    regular_split: token.regular_split,
                    closed: token.closed,
                    quoted: false,
                });
            }
        }
    }

    SearchInput {
        tokens: search_tokens,
        filter: search_filter,
    }
}

fn tokenize_input_query(q: &str) -> Vec::<InputToken> {
    let mut tokens = Vec::<InputToken>::new();

    // We don't care about unicode here since all split conditions
    // only involve ascii characters.
    let mut within_quotes = false;
    let mut alphabetic_counter = 0;
    let mut token_start = 0;
    for (i, c) in q.char_indices() {
        // Quote escaping is not supported
        if c == '"' {
            within_quotes = !within_quotes;

            // Only closing (even-numbered) quotes do a split (see below).
            // Opening quotes should not split (therefore the continue).
            if within_quotes {
                continue;
            }
        }

        // Note:
        // - Regular splits are splits based on a specific character and technically inclusive
        //   (for quotes at least, whitespace is trimmed),
        // - Irregular splits are splits based on a specific pattern and exclusive.
        //
        // It can happen that two splits need to be made:
        //   "physik hs1" on the last char
        //             ^
        // does an irregular split ("hs") and regular split ("1") because of
        // the end of the query string.
        //
        // For this reason irregular splits are determined before regular splits.

        // There is a special case when up to 3 alphabetic chars are followed by a numeric part.
        // This is intended to split up strings like "MW1250".
        if c.is_numeric() && 0 < alphabetic_counter && alphabetic_counter <= 3 {
            tokens.push(InputToken {
                s: q.get(token_start..=(i-1)).unwrap().trim_end().to_lowercase(),
                regular_split: false,
                closed: true,
            });

            token_start = i;
        }

        if (!within_quotes && c.is_whitespace() && i > token_start) ||  // whitespace
           ((within_quotes || !c.is_whitespace()) && i+1 == q.len()) ||  // end of string
           (c == '"') {  // end of quotes
            tokens.push(InputToken {
                // Note: The trim_end also trims within unclosed quotes at the end of the query,
                //       but currently I don't think this is an issue.
                s: q.get(token_start..=i).unwrap().trim_end().to_lowercase(),
                regular_split: true,
                // `closed` indicates whether the token has been closed (by whitespace or quote)
                // at the end, when this is the last token. This is relevant because MeiliSearch
                // treats whitespace at the end differently, and we might want to imitate that
                // behaviour.
                closed: !(i+1 == q.len() && (within_quotes || (c != '"' && !c.is_whitespace()))),
            });

            token_start = i + 1;
        } else if !within_quotes && c.is_whitespace() {
            //    ^
            // To avoid empty tokens when there are multiple whitespaces, we need
            // to move the token start even if there was no split.
            token_start = i + 1;
        }

        if c.is_alphabetic() {
            alphabetic_counter += 1;
        } else {
            alphabetic_counter = 0;
        }
    }

    tokens
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
