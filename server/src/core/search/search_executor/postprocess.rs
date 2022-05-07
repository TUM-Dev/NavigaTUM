use crate::core::search::search_executor::meilisearch::MSHit;
use crate::core::search::search_executor::preprocess::SearchToken;

pub(super) fn highlight_matches(s: &String, search_tokens: &Vec<SearchToken>) -> String {
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

// Parse the search against some known room formats and improve the
// results display in this case. Room formats are hardcoded for now.
pub(super) fn parse_room_formats(search_tokens: &Vec<SearchToken>, hit: &MSHit) -> Option<String> {
    // Some building specific roomcode formats are determined by their building prefix
    if search_tokens.len() == 2
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
            arch_id.get(search_tokens[1].s.len()..).unwrap(),
        ))
    }
    // If it doesn't match some precise room format, but the search is clearly
    // matching the arch name and not the main name, then we highlight this arch name.
    // This is intentionally still restrictive and considers only the first token,
    // because we expect searches for arch names not to start with anything else.
    else if (search_tokens.len() == 1
             || (search_tokens.len() > 1 && search_tokens[0].s.len() >= 3))
        //     Needs to be specific enough to be considered relevant ↑
        && !hit.name.contains(&search_tokens[0].s) // No match in the name
        && hit.parent_building.len() > 0 // Has parent information to show in query
        && hit.arch_name.is_some()
        && hit
            .arch_name
            .as_ref()
            .unwrap()
            .starts_with(&search_tokens[0].s)
    {
        // Exclude the part after the "@" if it's not in the query and use the
        // building name instead, because this is probably more helpful
        let (prefix, parsed_arch_id) = if search_tokens[0].s.contains("@") {
            (None, hit.arch_name.as_ref().unwrap().to_string())
        } else {
            let arch_id = hit.arch_name.as_ref().unwrap().split("@").next().unwrap();
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
