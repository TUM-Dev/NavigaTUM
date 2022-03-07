use futures::join;
use std::collections::HashMap;
use std::time::Instant;

use actix_web::client::{Client, ClientBuilder, Connector};
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Deserialize)]
pub struct SearchQueryArgs {
    // Limit per facet
    limit_buildings: Option<usize>,
    limit_rooms: Option<usize>,
    limit_all: Option<usize>,
}

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
    parent: Option<Vec<String>>,
    r#type: Option<Vec<String>>,
    usage: Option<Vec<String>>,
}

#[derive(Debug)]
struct SearchInput {
    tokens: Vec<SearchToken>,
    filter: SearchFilter,
}

/// Returned search results by this
#[derive(Serialize, Debug)]
pub struct SearchResults {
    sections: Vec<SearchResultsSection>,
    time_ms: u128,
}

#[derive(Serialize, Debug)]
pub struct SearchResultsSection {
    facet: String,
    entries: Vec<ResultEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_visible: Option<usize>,
    nb_hits: i32,
}

#[derive(Serialize, Debug)]
pub struct ResultEntry {
    id: String,
    r#type: String,
    name: String,
    subtext: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtext_bold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parsed_id: Option<String>,
}

/// Input into MeiliSearch
struct MSSearchArgs<'a> {
    q: &'a str,
    filter: Option<MSSearchFilter>,
    limit: u8,
}

#[derive(Debug)]
struct MSSearchFilter {
    facet: Vec<String>,
}

#[derive(Serialize)]
struct MSQuery<'a> {
    q: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Vec<Vec<String>>>,
    limit: u8,
    #[serde(rename = "facetsDistribution")]
    facets_distribution: Vec<String>,
}

/// Result format of MeiliSearch.
#[derive(Deserialize)]
#[allow(dead_code)]
struct MSResults {
    hits: Vec<MSHit>,
    #[serde(rename = "nbHits")]
    nb_hits: i32,
    #[serde(rename = "facetsDistribution")]
    facets_distribution: MSFacetDistribution,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct MSHit {
    ms_id: String,
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

#[derive(Deserialize, Debug)]
struct MSFacetDistribution {
    facet: HashMap<String, i32>,
}

pub async fn do_search(q: String, args: SearchQueryArgs) -> Result<SearchResults> {
    let start_time = Instant::now();

    let parsed = parse_input_query(&q);

    let client = ClientBuilder::new()
        .connector(Connector::new().finish())
        .finish();

    let results_sections = do_geoentry_search(client, &parsed.tokens, args).await;

    let time_ms = start_time.elapsed().as_millis();
    Ok(SearchResults {
        sections: results_sections,
        time_ms,
    })
}

fn build_query_string(search_tokens: &Vec<SearchToken>) -> String {
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

                    let v = token
                        .s
                        .trim_start_matches(prefix)
                        .trim_start_matches("\"")
                        .trim_end_matches("\"");
                    if v.len() == 0 {
                        // e.g. ' in: ', ' @ ', ' in:"" ' are ignored,
                        continue; // TODO: autosuggest
                    };

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

fn tokenize_input_query(q: &str) -> Vec<InputToken> {
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
                s: q.get(token_start..=(i - 1))
                    .unwrap()
                    .trim_end()
                    .to_lowercase(),
                regular_split: false,
                closed: true,
            });

            token_start = i;
        }

        if (!within_quotes && c.is_whitespace() && i > token_start) ||  // whitespace
           ((within_quotes || !c.is_whitespace()) && i+c.len_utf8() == q.len()) ||  // end of string
           (c == '"')
        {
            // end of quotes
            tokens.push(InputToken {
                // Note: The trim_end also trims within unclosed quotes at the end of the query,
                //       but currently I don't think this is an issue.
                s: q.get(token_start..i + c.len_utf8())
                    .unwrap()
                    .trim_end()
                    .to_lowercase(),
                regular_split: true,
                // `closed` indicates whether the token has been closed (by whitespace or quote)
                // at the end, when this is the last token. This is relevant because MeiliSearch
                // treats whitespace at the end differently, and we might want to imitate that
                // behaviour.
                closed: !(i + c.len_utf8() == q.len()
                    && (within_quotes || (c != '"' && !c.is_whitespace()))),
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

async fn do_geoentry_search(
    client: Client,
    search_tokens: &Vec<SearchToken>,
    args: SearchQueryArgs,
) -> Vec<SearchResultsSection> {
    // Determine what to search for

    // Currently ranking is designed to put buildings at the top if they equally
    // match the term compared to a room. For this reason there is only a search
    // for all entries and only rooms, as matching (and relevant) buildings can be
    // expected to be at the top of the merged search. However sometimes a lot of
    // buildings will be hidden (e.g. building parts), so the extra room search ....

    let q_default = build_query_string(&search_tokens);

    let res_merged = do_meilisearch(
        client.clone(),
        MSSearchArgs {
            q: &q_default,
            filter: None,
            limit: args.limit_all.unwrap_or(20) as u8, // This is the MeiliSearch default
        },
    );
    // Building limit multiplied by two because we might do reordering later
    let res_buildings = do_building_search_closed_query(
        client.clone(),
        &search_tokens,
        2 * args.limit_buildings.unwrap_or(5) as u8,
    );
    let res_rooms = do_room_search(
        client.clone(),
        &search_tokens,
        args.limit_rooms.unwrap_or(5) as u8,
    );

    let results = join!(res_merged, res_buildings, res_rooms);

    // First look up which buildings did match even with a closed query.
    // We can consider them more relevant.
    let mut closed_matching_buildings = Vec::<String>::new();
    for hit in results.1.unwrap().hits {
        closed_matching_buildings.push(hit.id);
    }

    let mut section_buildings = SearchResultsSection {
        facet: "sites_buildings".to_string(),
        entries: Vec::<ResultEntry>::new(),
        n_visible: None,
        nb_hits: results
            .0
            .as_ref()
            .unwrap()
            .facets_distribution
            .facet
            .get("site")
            .unwrap_or_else(|| &0)
            + results
                .0
                .as_ref()
                .unwrap()
                .facets_distribution
                .facet
                .get("building")
                .unwrap_or_else(|| &0),
    };
    let mut section_rooms = SearchResultsSection {
        facet: "rooms".to_string(),
        entries: Vec::<ResultEntry>::new(),
        n_visible: None,
        nb_hits: results.2.as_ref().unwrap().nb_hits,
    };

    // TODO: Collapse joined buildings
    // let mut observed_joined_buildings = Vec::<String>::new();
    let mut observed_ids = Vec::<String>::new();
    for hit in [results.0.unwrap().hits, results.2.unwrap().hits].concat() {
        if observed_ids.contains(&hit.id) {
            continue;
        }; // No duplicates

        // Find out where it matches TODO: Improve
        let highlighted_name = highlight_matches(&hit.name, &search_tokens);
        let highlighted_arch_name = match &hit.arch_name {
            Some(arch_name) => highlight_matches(arch_name, &search_tokens),
            None => String::from(""),
        };

        match hit.r#type.as_str() {
            "campus" | "site" | "area" | "building" | "joined_building" => {
                if section_buildings.entries.len() < args.limit_buildings.unwrap_or(5) {
                    section_buildings.entries.push(ResultEntry {
                        id: hit.id.to_string(),
                        r#type: hit.r#type.to_string(),
                        name: highlighted_name,
                        subtext: format!("{}", hit.type_common_name),
                        subtext_bold: None,
                        parsed_id: None,
                    });
                }
            }
            "room" | "virtual_room" => {
                if section_rooms.entries.len() < args.limit_rooms.unwrap_or(5)
                    || (section_rooms.entries.len()
                        < (args.limit_rooms.unwrap_or(5) + args.limit_buildings.unwrap_or(5))
                        && section_buildings.entries.len() == 0)
                {
                    // Test whether the query matches some common room id formats.
                    // This is hardcoded here for now and should be changed in the future.
                    let parsed_id = if search_tokens.len() == 2
                        && match search_tokens[0].s.as_str() {
                            "mi" => hit.id.starts_with("560") || hit.id.starts_with("561"),
                            "mw" => hit.id.starts_with("550") || hit.id.starts_with("551"),
                            "ph" => hit.id.starts_with("5101"),
                            "ch" => hit.id.starts_with("540"),
                            _ => false,
                        }
                        && !search_tokens[1].s.contains("@")
                        && hit.arch_name.is_some()
                        && hit
                            .arch_name
                            .as_ref()
                            .unwrap()
                            .starts_with(&search_tokens[1].s)
                    {
                        let arch_id = hit.arch_name.as_ref().unwrap().split("@").next().unwrap();
                        Some(format!(
                            "\u{0019}{} {}\u{0017}{}",
                            search_tokens[0].s.to_uppercase(),
                            arch_id.get(..search_tokens[1].s.len()).unwrap(),
                            arch_id.get(search_tokens[1].s.len()..).unwrap_or_default(),
                        ))
                    } else {
                        None
                    };

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
                            Some(hit.arch_name.unwrap_or_default())
                        } else {
                            Some(highlighted_arch_name)
                        },
                        parsed_id: parsed_id,
                    });

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

async fn do_building_search_closed_query(
    client: Client,
    search_tokens: &Vec<SearchToken>,
    limit: u8,
) -> Result<MSResults> {
    let q = format!("{} ", build_query_string(search_tokens));

    do_meilisearch(
        client,
        MSSearchArgs {
            q: &q,
            filter: Some(MSSearchFilter {
                facet: vec!["site".to_string(), "building".to_string()],
            }),
            limit: limit,
        },
    )
    .await
}

async fn do_room_search(
    client: Client,
    search_tokens: &Vec<SearchToken>,
    limit: u8,
) -> Result<MSResults> {
    let mut q = String::from("");
    for token in search_tokens {
        // It is common that room names are given with four-digits with the first digit
        // being the level. In this case we add splitted terms as well, which could give
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
            limit: limit,
        },
    )
    .await
}

async fn do_meilisearch(client: Client, args: MSSearchArgs<'_>) -> Result<MSResults> {
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

fn highlight_matches(s: &String, search_tokens: &Vec<SearchToken>) -> String {
    // Note: This does not highlight the matches that were actually used.
    //       e.g. "hs" will highlight "Versuchsraum"
    //                                       ^^
    // TODO: This could in some cases be misleading
    let mut s_highlighted = s.to_string();

    for token in search_tokens {
        let s_lower = s_highlighted.to_lowercase();
        let mut offset = 0;
        for (start_i, pattern) in s_lower.match_indices(&token.s) {
            let highlight_start = start_i + offset;
            let highlight_end = highlight_start + pattern.len();
            if start_i > 0
                && s_highlighted
                    .get(..highlight_start)
                    .unwrap()
                    .chars()
                    .last()
                    .unwrap()
                    .is_alphabetic()
                && pattern.chars().next().unwrap().is_alphabetic()
            {
                continue;
            }

            let pre_highlight = s_highlighted.get(..highlight_start).unwrap();
            let highlight = s_highlighted.get(highlight_start..highlight_end).unwrap();
            let post_highlight = s_highlighted.get(highlight_end..).unwrap();
            s_highlighted = format!(
                "{}\u{0019}{}\u{0017}{}",
                pre_highlight, highlight, post_highlight
            );
            offset += 2;
        }
    }

    s_highlighted.to_string()
}
