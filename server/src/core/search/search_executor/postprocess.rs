use super::meilisearch;
use super::preprocess;

pub(super) fn merge_search_results(
    args: &super::SanitisedSearchQueryArgs,
    search_tokens: &Vec<preprocess::SearchToken>,
    res_merged: meilisearch::MSResults,
    res_buildings: meilisearch::MSResults,
    res_rooms: meilisearch::MSResults,
) -> Vec<super::SearchResultsSection> {
    // First look up which buildings did match even with a closed query.
    // We can consider them more relevant.
    // TODO: This has to be implemented. closed_matching_buildings is not used further down in this function.
    let mut closed_matching_buildings = Vec::<String>::new();
    for hit in res_buildings.hits {
        closed_matching_buildings.push(hit.id);
    }

    let facet = res_merged.facet_distribution.facet;
    let mut section_buildings = super::SearchResultsSection {
        facet: "sites_buildings".to_string(),
        entries: Vec::<super::ResultEntry>::new(),
        n_visible: None,
        estimated_total_hits: facet.get("site").unwrap_or(&0) + facet.get("building").unwrap_or(&0),
    };
    let mut section_rooms = super::SearchResultsSection {
        facet: "rooms".to_string(),
        entries: Vec::<super::ResultEntry>::new(),
        n_visible: None,
        estimated_total_hits: res_rooms.estimated_total_hits,
    };

    // TODO: Collapse joined buildings
    // let mut observed_joined_buildings = Vec::<String>::new();
    let mut observed_ids = Vec::<String>::new();
    for hit in [res_merged.hits, res_rooms.hits].concat() {
        // Prevent duplicates from being added to the results
        if observed_ids.contains(&hit.id) {
            continue;
        };
        observed_ids.push(hit.id.clone());

        // Total limit reached (does only count visible results)
        let current_buildings_cnt = section_buildings
            .n_visible
            .unwrap_or(section_buildings.entries.len());
        if section_rooms.entries.len() + current_buildings_cnt >= args.limit_all {
            break;
        }

        // Find out where it matches TODO: Improve
        let highlighted_name = highlight_matches(&hit.name, search_tokens);
        let highlighted_arch_name = match &hit.arch_name {
            Some(arch_name) => highlight_matches(arch_name, search_tokens),
            None => String::from(""),
        };

        match hit.r#type.as_str() {
            "campus" | "site" | "area" | "building" | "joined_building" => {
                if section_buildings.entries.len() < args.limit_buildings {
                    push_to_buildings_queue(&mut section_buildings, &hit, highlighted_name);
                }
            }
            "room" | "virtual_room" => {
                if section_rooms.entries.len() < args.limit_rooms {
                    push_to_rooms_queue(
                        &mut section_rooms,
                        &hit,
                        search_tokens,
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
    }

    match section_buildings.n_visible {
        Some(0) => vec![section_rooms, section_buildings],
        _ => vec![section_buildings, section_rooms],
    }
}

fn push_to_buildings_queue(
    section_buildings: &mut super::SearchResultsSection,
    hit: &meilisearch::MSHit,
    highlighted_name: String,
) {
    section_buildings.entries.push(super::ResultEntry {
        id: hit.id.to_string(),
        r#type: hit.r#type.to_string(),
        name: highlighted_name,
        subtext: hit.type_common_name.clone(),
        subtext_bold: None,
        parsed_id: None,
    });
}

fn push_to_rooms_queue(
    section_rooms: &mut super::SearchResultsSection,
    hit: &meilisearch::MSHit,
    search_tokens: &[preprocess::SearchToken],
    highlighted_name: String,
    highlighted_arch_name: String,
) {
    // Test whether the query matches some common room id formats
    let parsed_id = parse_room_formats(search_tokens, hit);

    let subtext = match hit.parent_building.len() {
        0 => String::from(""),
        _ => hit.parent_building[0].clone(),
    };
    let subtext_bold = match parsed_id {
        Some(_) => Some(hit.arch_name.clone().unwrap_or_default()),
        None => Some(highlighted_arch_name),
    };
    section_rooms.entries.push(super::ResultEntry {
        id: hit.id.to_string(),
        r#type: hit.r#type.to_string(),
        name: highlighted_name,
        subtext,
        subtext_bold,
        parsed_id,
    });
}

fn highlight_matches(s: &String, search_tokens: &Vec<preprocess::SearchToken>) -> String {
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

    s_highlighted
}

// Parse the search against some known room formats and improve the
// results display in this case. Room formats are hardcoded for now.
fn parse_room_formats(
    search_tokens: &[preprocess::SearchToken],
    hit: &meilisearch::MSHit,
) -> Option<String> {
    // Some building specific roomcode formats are determined by their building prefix
    if search_tokens.len() == 2
        && match search_tokens[0].s.as_str() {
            "mi" => hit.id.starts_with("560") || hit.id.starts_with("561"),
            "mw" => hit.id.starts_with("550") || hit.id.starts_with("551"),
            "ph" => hit.id.starts_with("5101"),
            "ch" => hit.id.starts_with("540"),
            _ => false,
        }
        && !search_tokens[1].s.contains('@')
        && hit.arch_name.is_some()
        && hit
            .arch_name
            .as_ref()
            .unwrap()
            .starts_with(&search_tokens[1].s)
    {
        let arch_id = hit.arch_name.as_ref().unwrap().split('@').next().unwrap();
        Some(format!(
            "\u{0019}{} {}\u{0017}{}",
            search_tokens[0].s.to_uppercase(),
            arch_id.get(..search_tokens[1].s.len()).unwrap(),
            arch_id.get(search_tokens[1].s.len()..).unwrap(),
        ))
    // If it doesn't match some precise room format, but the search is clearly
    // matching the arch name and not the main name, then we highlight this arch name.
    // This is intentionally still restrictive and considers only the first token,
    // because we expect searches for arch names not to start with anything else.
    } else if (search_tokens.len() == 1
             || (search_tokens.len() > 1 && search_tokens[0].s.len() >= 3))
        //     Needs to be specific enough to be considered relevant ↑
        && !hit.name.contains(&search_tokens[0].s) // No match in the name
        && !hit.parent_building.is_empty() // Has parent information to show in query
        && hit.arch_name.is_some()
        && hit
            .arch_name
            .as_ref()
            .unwrap()
            .starts_with(&search_tokens[0].s)
    {
        // Exclude the part after the "@" if it's not in the query and use the
        // building name instead, because this is probably more helpful
        let (prefix, parsed_arch_id) = if search_tokens[0].s.contains('@') {
            (None, hit.arch_name.as_ref().unwrap().to_string())
        } else {
            let arch_id = hit.arch_name.as_ref().unwrap().split('@').next().unwrap();
            // For some well known buildings we have a prefix that we can use instead
            let prefix = match hit.name.get(..3).unwrap_or_default() {
                "560" | "561" => Some("MI "),
                "550" | "551" => Some("MW "),
                "540" => Some("CH "),
                _ => match hit.name.get(..4).unwrap_or_default() {
                    "5101" => Some("PH "),
                    "5107" => Some("PH II "),
                    _ => None,
                },
            };
            if prefix.is_some() {
                (prefix, arch_id.to_string())
            } else if hit.parent_building[0].len() > 25 {
                // Building names can sometimes be quite long, which doesn't
                // look nice. Since this building name here serves only search a
                // hint, we'll crop it (with more from the end, because there
                // is usually more entropy)
                (
                    None,
                    format!(
                        "{} {}…{}",
                        arch_id,
                        hit.parent_building[0].get(..7).unwrap(),
                        hit.parent_building[0]
                            .get((hit.parent_building[0].len() - 10)..)
                            .unwrap()
                    ),
                )
            } else {
                (None, format!("{} {}", arch_id, hit.parent_building[0]))
            }
        };
        Some(format!(
            "{}\u{0019}{}\u{0017}{}",
            prefix.unwrap_or_default(),
            parsed_arch_id.get(..search_tokens[0].s.len()).unwrap(),
            parsed_arch_id
                .get(search_tokens[0].s.len()..)
                .unwrap_or_default(),
        ))
    } else {
        None
    }
}
