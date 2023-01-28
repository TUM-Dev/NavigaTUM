use super::meilisearch;
use super::preprocess;
use unicode_truncate::UnicodeTruncateStr;

pub(super) fn merge_search_results(
    args: &super::SanitisedSearchQueryArgs,
    search_tokens: &[preprocess::SearchToken],
    res_merged: meilisearch::MSResults,
    res_buildings: meilisearch::MSResults,
    res_rooms: meilisearch::MSResults,
    highlighting: (String, String),
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

        match hit.r#type.as_str() {
            "campus" | "site" | "area" | "building" | "joined_building" => {
                if section_buildings.entries.len() < args.limit_buildings {
                    push_to_buildings_queue(
                        &mut section_buildings,
                        hit.clone(),
                        hit._formatted.name,
                    );
                }
            }
            "room" | "virtual_room" => {
                if section_rooms.entries.len() < args.limit_rooms {
                    push_to_rooms_queue(
                        &mut section_rooms,
                        hit.clone(),
                        search_tokens,
                        hit._formatted.name,
                        hit.arch_name.unwrap_or_default(),
                        highlighting.clone(),
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
    hit: meilisearch::MSHit,
    highlighted_name: String,
) {
    section_buildings.entries.push(super::ResultEntry {
        id: hit.id.to_string(),
        r#type: hit.r#type,
        name: highlighted_name,
        subtext: hit.type_common_name,
        subtext_bold: None,
        parsed_id: None,
    });
}

fn push_to_rooms_queue(
    section_rooms: &mut super::SearchResultsSection,
    hit: meilisearch::MSHit,
    search_tokens: &[preprocess::SearchToken],
    formatted_name: String,
    arch_name: String,
    highlighting: (String, String),
) {
    // Test whether the query matches some common room id formats
    let parsed_id = parse_room_formats(search_tokens, &hit, &highlighting);

    let subtext = generate_subtext(&hit);
    let subtext_bold = match parsed_id {
        Some(_) => Some(hit.arch_name.clone().unwrap_or_default()),
        None => Some(arch_name),
    };
    section_rooms.entries.push(super::ResultEntry {
        id: hit.id.to_string(),
        r#type: hit.r#type,
        name: formatted_name,
        subtext,
        subtext_bold,
        parsed_id,
    });
}

// Parse the search against some known room formats and improve the
// results display in this case. Room formats are hardcoded for now.
fn parse_room_formats(
    search_tokens: &[preprocess::SearchToken],
    hit: &meilisearch::MSHit,
    highlighting: &(String, String),
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
        let split_arch_id = unicode_split_at(arch_id, search_tokens[1].s.len());
        Some(format!(
            "{}{} {}{}{}",
            highlighting.0,
            search_tokens[0].s.to_uppercase(),
            split_arch_id.0,
            highlighting.1,
            split_arch_id.1,
        ))
    // If it doesn't match some precise room format, but the search is clearly
    // matching the arch name and not the main name, then we highlight this arch name.
    // This is intentionally still restrictive and considers only the first token,
    // because we expect searches for arch names not to start with anything else.
    } else if (search_tokens.len() == 1
             || (search_tokens.len() > 1 && search_tokens[0].s.len() >= 3))
        //     Needs to be specific enough to be considered relevant ↑
        && !hit.unformatted_name.contains(&search_tokens[0].s) // No match in the name
        && !hit.parent_building_names.is_empty() // Has parent information to show in query
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
            let prefix = match hit.unformatted_name.unicode_truncate(3).0 {
                "560" | "561" => Some("MI "),
                "550" | "551" => Some("MW "),
                "540" => Some("CH "),
                _ => match hit.unformatted_name.unicode_truncate(4).0 {
                    "5101" => Some("PH "),
                    "5107" => Some("PH II "),
                    _ => None,
                },
            };
            if prefix.is_some() {
                (prefix, arch_id.to_string())
            } else if hit.parent_building_names[0].len() > 25 {
                // Building names can sometimes be quite long, which doesn't
                // look nice. Since this building name here serves only search a
                // hint, we'll crop it (with more from the end, because there
                // is usually more entropy)
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
        };
        let parsed_aid = unicode_split_at(&parsed_arch_id, search_tokens[0].s.len());
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

fn unicode_split_at(search: &str, width: usize) -> (&str, &str) {
    (
        search.unicode_truncate(width).0,
        search.unicode_truncate(search.len() - width).0,
    )
}

fn generate_subtext(hit: &meilisearch::MSHit) -> String {
    let building = match hit.parent_building_names.len() {
        0 => String::from(""),
        _ => hit.parent_building_names[0].clone(),
    };

    match &hit.campus {
        Some(campus) => format!("{campus}, {building}"),
        None => building,
    }
}
