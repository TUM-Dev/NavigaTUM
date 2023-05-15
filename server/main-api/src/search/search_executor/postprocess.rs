use crate::search::search_executor::parser::ParsedQuery;
use crate::search::search_executor::query::MSHit;
use meilisearch_sdk::search::{SearchResult, SearchResults};
use unicode_truncate::UnicodeTruncateStr;

pub(super) fn merge_search_results(
    args: &super::SanitisedSearchQueryArgs,
    res_merged: &SearchResults<MSHit>,
    res_buildings: &SearchResults<MSHit>,
    res_rooms: &SearchResults<MSHit>,
) -> (super::SearchResultsSection, super::SearchResultsSection) {
    // First look up which buildings did match even with a closed query.
    // We can consider them more relevant.
    // TODO: This has to be implemented. closed_matching_buildings is not used further down in this function.
    let mut closed_matching_buildings = Vec::<String>::new();
    for hit in &res_buildings.hits {
        closed_matching_buildings.push(hit.result.id.clone());
    }

    let mut section_buildings = super::SearchResultsSection {
        facet: "sites_buildings".to_string(),
        entries: Vec::new(),
        n_visible: None,
        estimated_total_hits: res_buildings.estimated_total_hits.unwrap_or(0),
    };
    let mut section_rooms = super::SearchResultsSection {
        facet: "rooms".to_string(),
        entries: Vec::new(),
        n_visible: None,
        estimated_total_hits: res_rooms.estimated_total_hits.unwrap_or(0),
    };

    // TODO: Collapse joined buildings
    // let mut observed_joined_buildings = Vec::<String>::new();
    let mut observed_ids = Vec::<String>::new();
    for hits in [&res_merged.hits, &res_rooms.hits] {
        for hit in hits.iter() {
            // Prevent duplicates from being added to the results
            if observed_ids.contains(&hit.result.id) {
                continue;
            };
            observed_ids.push(hit.result.id.clone());

            // Total limit reached (does only count visible results)
            let current_buildings_cnt = section_buildings
                .n_visible
                .unwrap_or(section_buildings.entries.len());
            if section_rooms.entries.len() + current_buildings_cnt >= args.limit_all {
                break;
            }
            let formatted_name = extract_formatted_name(hit).unwrap_or(hit.result.name.clone());

            let hit = hit.result.clone();
            match hit.r#type.as_str() {
                "campus" | "site" | "area" | "building" | "joined_building" => {
                    if section_buildings.entries.len() < args.limit_buildings {
                        section_buildings.entries.push(super::ResultEntry {
                            hit: hit.clone(),
                            id: hit.id.to_string(),
                            r#type: hit.r#type,
                            name: formatted_name,
                            subtext: hit.type_common_name,
                            subtext_bold: None,
                            parsed_id: None,
                        });
                    }
                }
                "room" | "virtual_room" => {
                    if section_rooms.entries.len() < args.limit_rooms {
                        section_rooms.entries.push(super::ResultEntry {
                            hit: hit.clone(),
                            id: hit.id.to_string(),
                            r#type: hit.r#type,
                            name: formatted_name,
                            subtext_bold: Some(hit.arch_name.unwrap_or_default()),
                            ..super::ResultEntry::default()
                        });

                        // The first room in the results 'freezes' the number of visible buildings
                        if section_buildings.n_visible.is_none() && section_rooms.entries.len() == 1
                        {
                            section_buildings.n_visible = Some(section_buildings.entries.len());
                        }
                    }
                }
                _ => {}
            };
        }
    }

    (section_buildings, section_rooms)
}

fn extract_formatted_name(hit: &SearchResult<MSHit>) -> Option<String> {
    Some(
        hit.formatted_result
            .clone()? //I don't understand why this is needed, but the performance impact is minimal
            .get("name")?
            .as_str()?
            .to_string(),
    )
}

// Parse the search against some known room formats and improve the
// results display in this case. Room formats are hardcoded for now.
pub(super) fn parse_room_formats(
    search_tokens: &ParsedQuery,
    hit: &MSHit,
    highlighting: &(String, String),
) -> Option<String> {
    // Some building specific roomcode formats are determined by their building prefix
    if search_tokens.len() == 2
        && match search_tokens[0].as_str() {
            "mi" => hit.id.starts_with("560") || hit.id.starts_with("561"),
            "mw" => hit.id.starts_with("550") || hit.id.starts_with("551"),
            "ph" => hit.id.starts_with("5101"),
            "ch" => hit.id.starts_with("540"),
            _ => false,
        }
        && !search_tokens[1].contains('@')
        && hit.arch_name.is_some()
        && hit
            .arch_name
            .as_ref()
            .unwrap()
            .starts_with(&search_tokens[1])
    {
        let arch_id = hit.arch_name.as_ref().unwrap().split('@').next().unwrap();
        let split_arch_id = unicode_split_at(arch_id, search_tokens[1].chars().count());
        Some(format!(
            "{}{} {}{}{}",
            highlighting.0,
            search_tokens[0].to_uppercase(),
            split_arch_id.0,
            highlighting.1,
            split_arch_id.1,
        ))
    // If it doesn't match some precise room format, but the search is clearly
    // matching the arch name and not the main name, then we highlight this arch name.
    // This is intentionally still restrictive and considers only the first token,
    // because we expect searches for arch names not to start with anything else.
    } else if (search_tokens.len() == 1
             || (search_tokens.len() > 1 && search_tokens[0].len() >= 3))
        //     Needs to be specific enough to be considered relevant ↑
        && !hit.name.contains(&search_tokens[0]) // No match in the name
        && !hit.parent_building_names.is_empty() // Has parent information to show in query
        && hit.arch_name.is_some()
        && hit
            .arch_name
            .as_ref()
            .unwrap()
            .starts_with(&search_tokens[0])
    {
        let (prefix, parsed_arch_id) = split_prefix_from_arch_building_id(search_tokens, &hit);
        let parsed_aid = unicode_split_at(&parsed_arch_id, search_tokens[0].chars().count());
        Some(format!(
            "{}{}{}{}{}",
            prefix.unwrap_or_default(),
            highlighting.0,
            parsed_aid.0,
            highlighting.1,
            parsed_aid.1,
        ))
    } else {
        None
    }
}

/// Exclude the part after the "@" if it's not in the query and use the
/// building name instead, because this is probably more helpful
fn split_prefix_from_arch_building_id<'a>(
    search_tokens: &ParsedQuery,
    hit: &&MSHit,
) -> (Option<&'a str>, String) {
    if search_tokens[0].contains('@') {
        return (None, hit.arch_name.as_ref().unwrap().to_string());
    }
    let arch_id = hit.arch_name.as_ref().unwrap().split('@').next().unwrap();
    // For some well known buildings we have a prefix that we can use instead
    let prefix = match unicode_split_at(&hit.name, 3).0 {
        "560" | "561" => Some("MI "),
        "550" | "551" => Some("MW "),
        "540" => Some("CH "),
        _ => match unicode_split_at(&hit.name, 4).0 {
            "5101" => Some("PH "),
            "5107" => Some("PH II "),
            _ => None,
        },
    };
    if prefix.is_some() {
        return (prefix, arch_id.to_string());
    }

    // Building names can sometimes be quite long, which doesn't
    // look nice. Since this building name here serves only search a
    // hint, we'll crop it (with more from the end, because there
    // is usually more entropy)
    if hit.parent_building_names[0].len() > 25 {
        let pn = hit.parent_building_names[0].as_str();
        let (first, _) = pn.unicode_truncate(7);
        let (last, _) = pn.unicode_truncate_start(10);
        (None, format!("{arch_id} {first}…{last}"))
    } else {
        (
            None,
            format!("{} {}", arch_id, hit.parent_building_names[0]),
        )
    }
}

pub(super) fn generate_subtext(hit: &MSHit) -> String {
    let building = match hit.parent_building_names.len() {
        0 => String::from(""),
        _ => hit.parent_building_names[0].clone(),
    };

    match &hit.campus {
        Some(campus) => format!("{campus}, {building}"),
        None => building,
    }
}

fn unicode_split_at(search: &str, width: usize) -> (&str, &str) {
    // since some UTF-8 grapheme clusters are more than one byte, we need to check where we can split
    let splitpoint = search.chars().take(width).collect::<String>().len();
    search.split_at(splitpoint)
}

#[cfg(test)]
mod postprocessing_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn unicode_split() {
        assert_eq!(unicode_split_at("Griaß eich", 4), ("Gria", "ß eich"));
        assert_eq!(unicode_split_at("Griaß eich", 5), ("Griaß", " eich"));
        assert_eq!(unicode_split_at("Tschöö", 4), ("Tsch", "öö"));
        assert_eq!(unicode_split_at("Tschöö", 5), ("Tschö", "ö"));
        assert_eq!(unicode_split_at("Tschöö", 6), ("Tschöö", ""));
        assert_eq!(unicode_split_at("Ähh", 0), ("", "Ähh"));
        assert_eq!(unicode_split_at("Ähh", 1), ("Ä", "hh"));
    }
}
