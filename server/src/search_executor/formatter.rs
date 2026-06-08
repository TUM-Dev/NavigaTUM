use unicode_truncate::UnicodeTruncateStr as _;

use super::ResultEntry;
use super::parser::{ParsedQuery, TextToken};
use crate::external::meilisearch::{GeoMSHit, MSHit};
use crate::routes::search::{CroppingMode, FormattingConfig, ParsedIdMode};

pub(super) struct RoomVisitor {
    parsed_input: ParsedQuery,
    config: FormattingConfig,
}

impl From<(ParsedQuery, FormattingConfig)> for RoomVisitor {
    #[tracing::instrument]
    fn from((parsed_input, config): (ParsedQuery, FormattingConfig)) -> Self {
        Self {
            parsed_input,
            config,
        }
    }
}

impl RoomVisitor {
    pub(super) fn visit(&self, item: &mut ResultEntry) {
        // Only geo (room) hits carry the arch-name / parent metadata this
        // visitor formats. The visitor is only ever run over the rooms
        // section, so a non-geo hit here would be a bug, but we degrade
        // gracefully rather than panic.
        let MSHit::Geo(hit) = &item.hit else {
            return;
        };
        match self.config.parsed_id {
            ParsedIdMode::Prefixed => {
                item.parsed_id = self.parse_room_formats(hit);
            }
            ParsedIdMode::Roomfinder => {
                item.parsed_id = hit.arch_name.clone();
            }
        }
        item.subtext = Self::generate_subtext(hit);
    }
    // Parse the search against some known room formats and improve the
    // results display in this case. Room formats are hardcoded for now.
    fn parse_room_formats(&self, hit: &GeoMSHit) -> Option<String> {
        let first_token = self.parsed_input.tokens.first()?;
        let archname = hit.arch_name.clone()?;
        match first_token {
            TextToken::SplittableText((t0, t1))
                if self.parsed_input.relevant_enough_for_room_highligting() =>
            {
                let building_specific_roomcode_format_determinable_by_building_prefix = match t0
                    .as_str()
                {
                    "mi" => hit.room_code.starts_with("560") || hit.room_code.starts_with("561"),
                    "mw" => hit.room_code.starts_with("550") || hit.room_code.starts_with("551"),
                    "ph" => hit.room_code.starts_with("5101"),
                    "ch" => hit.room_code.starts_with("540"),
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
                    self.config.highlighting.pre,
                    t0.to_uppercase(),
                    split_arch_id.0,
                    self.config.highlighting.post,
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

                let (prefix, parsed_arch_id) =
                    Self::split_prefix_from_arch_building_id(hit, text, &self.config);
                let parsed_aid = unicode_split_at(&parsed_arch_id, text.chars().count());
                Some(format!(
                    "{}{}{}{}{}",
                    prefix.unwrap_or_default(),
                    self.config.highlighting.pre,
                    parsed_aid.0,
                    self.config.highlighting.post,
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
        hit: &GeoMSHit,
        first_token: &str,
        config: &FormattingConfig,
    ) -> (Option<&'a str>, String) {
        if first_token.contains('@') {
            return (
                None,
                hit.arch_name
                    .as_ref()
                    .expect("caller guarantees arch_name is set when first_token contains '@'")
                    .clone(),
            );
        }
        let arch_id = hit
            .arch_name
            .as_ref()
            .expect("caller guarantees arch_name is set for room hits")
            .split('@')
            .next()
            .expect("split iterator always yields at least one element");
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
        // e.g. "560-561 Hauptgebäude der Fakultät für Informatik und Mathematik"
        // becomes "560-561 Hauptgebäude…Mathematik"
        // crop only when explicitly configured (default is Crop)
        let building_name = hit.parent_building_names.first().map_or("", String::as_str);
        if config.cropping == CroppingMode::Crop && building_name.len() > 25 {
            let (first, _) = building_name.unicode_truncate(7);
            let (last, _) = building_name.unicode_truncate_start(10);
            (None, format!("{arch_id} {first}…{last}"))
        } else if building_name.is_empty() {
            (None, arch_id.to_string())
        } else {
            (None, format!("{arch_id} {building_name}"))
        }
    }

    fn generate_subtext(hit: &GeoMSHit) -> String {
        let building = hit
            .parent_building_names
            .first()
            .cloned()
            .unwrap_or_default();

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
