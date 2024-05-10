use crate::search::search_executor::parser::{ParsedQuery, TextToken};
use crate::search::search_executor::query::MSHit;
use crate::search::search_executor::ResultEntry;
use unicode_truncate::UnicodeTruncateStr;

pub(super) struct RoomVisitor {
    parsed_input: ParsedQuery,
    highlighting: (String, String),
}

impl RoomVisitor {
    pub(super) const fn from(parsed_input: ParsedQuery, highlighting: (String, String)) -> Self {
        Self {
            parsed_input,
            highlighting,
        }
    }
    pub(super) fn visit(&self, item: &mut ResultEntry) {
        item.parsed_id = self.parse_room_formats(&item.hit);
        item.subtext = Self::generate_subtext(&item.hit);
    }
    // Parse the search against some known room formats and improve the
    // results display in this case. Room formats are hardcoded for now.
    fn parse_room_formats(&self, hit: &MSHit) -> Option<String> {
        let first_token = self.parsed_input.tokens.first()?;
        let archname = hit.arch_name.clone()?;
        match first_token {
            TextToken::SplittableText((t0, t1))
                if self.parsed_input.relevant_enough_for_room_highligting() =>
            {
                let building_specific_roomcode_format_determinable_by_building_prefix =
                    match t0.as_str() {
                        "mi" => hit.id.starts_with("560") || hit.id.starts_with("561"),
                        "mw" => hit.id.starts_with("550") || hit.id.starts_with("551"),
                        "ph" => hit.id.starts_with("5101"),
                        "ch" => hit.id.starts_with("540"),
                        _ => false,
                    };
                if !building_specific_roomcode_format_determinable_by_building_prefix
                    || !archname.starts_with(t1)
                {
                    return None;
                }
                let arch_id = archname.split('@').next()?;
                let split_arch_id = unicode_split_at(arch_id, t0.chars().count());
                Some(format!(
                    "{}{} {}{}{}",
                    self.highlighting.0,
                    t0.to_uppercase(),
                    split_arch_id.0,
                    self.highlighting.1,
                    split_arch_id.1,
                ))
            }

            // If it doesn't match some precise room format, but the search is clearly
            // matching the arch name and not the main name, then we highlight this arch name.
            // This is intentionally still restrictive and considers only the first token,
            // because we expect searches for arch names not to start with anything else.
            TextToken::Text(text) if self.parsed_input.relevant_enough_for_room_highligting() => {
                let no_parent_information_to_show_in_query = hit.parent_building_names.is_empty();
                if hit.name.contains(text)
                    || no_parent_information_to_show_in_query
                    || !archname.starts_with(text)
                {
                    return None;
                }

                let (prefix, parsed_arch_id) = Self::split_prefix_from_arch_building_id(hit, text);
                let parsed_aid = unicode_split_at(&parsed_arch_id, text.chars().count());
                Some(format!(
                    "{}{}{}{}{}",
                    prefix.unwrap_or_default(),
                    self.highlighting.0,
                    parsed_aid.0,
                    self.highlighting.1,
                    parsed_aid.1,
                ))
            }

            // not relevant enough for room highlighting
            TextToken::Text(_) | TextToken::SplittableText(_) => None,
        }
    }

    /// Exclude the part after the "@" if it's not in the query and use the
    /// building name instead, because this is probably more helpful
    fn split_prefix_from_arch_building_id<'a>(
        hit: &MSHit,
        first_token: &str,
    ) -> (Option<&'a str>, String) {
        if first_token.contains('@') {
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

    fn generate_subtext(hit: &MSHit) -> String {
        let building = match hit.parent_building_names.len() {
            0 => String::new(),
            _ => hit.parent_building_names[0].clone(),
        };

        match &hit.campus {
            Some(campus) => format!("{campus}, {building}"),
            None => building,
        }
    }
}

fn unicode_split_at(search: &str, width: usize) -> (&str, &str) {
    // since some UTF-8 grapheme clusters are more than one byte, we need to check where we can split
    let splitpoint = search.chars().take(width).collect::<String>().len();
    search.split_at(splitpoint)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

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
