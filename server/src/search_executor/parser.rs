use logos::Logos;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use tracing::warn;

use super::lexer::Token;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Filter {
    parents: HashSet<String>,
    types: HashSet<String>,
    usages: HashSet<String>,
}
impl Filter {
    pub fn as_meilisearch_filters(&self) -> String {
        let mut filters = vec![];
        if !self.parents.is_empty() {
            let parents: Vec<&str> = self.parents.iter().map(String::as_str).collect();
            filters.push(format!(
                "((parent_keywords IN {parents:?}) OR (parent_building_names IN {parents:?}) OR (campus IN {parents:?}))"
            ));
        }
        if !self.types.is_empty() {
            let types: Vec<&str> = self.types.iter().map(String::as_str).collect();
            filters.push(format!("(type IN {types:?})"));
        }
        if !self.usages.is_empty() {
            let usages: Vec<&str> = self.usages.iter().map(String::as_str).collect();
            filters.push(format!("(usage IN {usages:?})"));
        }
        filters.join(" AND ")
    }
    pub fn is_empty(&self) -> bool {
        self.parents.is_empty() && self.types.is_empty() && self.usages.is_empty()
    }
}

impl Debug for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_struct("Filter");
        if !self.parents.is_empty() {
            base.field("parents", &self.parents);
        }
        if !self.types.is_empty() {
            base.field("types", &self.parents);
        }
        if !self.usages.is_empty() {
            base.field("usages", &self.parents);
        }
        base.finish()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sorting {
    location: HashSet<String>,
}

impl Sorting {
    pub fn as_meilisearch_sorting(&self) -> Vec<String> {
        self.location
            .iter()
            .map(|s| format!("_geoPoint({s}):asc"))
            .collect()
    }
    pub fn is_empty(&self) -> bool {
        self.location.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextToken {
    Text(String),
    SplittableText((String, String)),
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ParsedQuery {
    pub tokens: Vec<TextToken>,
    pub filters: Filter,
    pub sorting: Sorting,
}

impl Debug for ParsedQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_struct("ParsedQuery");
        base.field("tokens", &self.tokens);
        if !self.filters.is_empty() {
            base.field("from", &self.filters);
        }
        if !self.sorting.is_empty() {
            base.field("from", &self.sorting);
        }
        base.finish()
    }
}

impl ParsedQuery {
    pub fn relevant_enough_for_room_highligting(&self) -> bool {
        if self.tokens.len() == 1 {
            return true;
        }
        match self.tokens.first() {
            Some(first) => match first {
                TextToken::Text(t) => t.len() > 3,
                TextToken::SplittableText((t0, _)) => t0.len() > 3,
            },
            None => false,
        }
    }
}

impl From<&str> for ParsedQuery {
    fn from(query: &str) -> Self {
        let mut result = Self::default();
        for token in Token::lexer(query) {
            match token {
                Ok(Token::Text(s)) => {
                    result.tokens.push(TextToken::Text(s));
                }
                Ok(Token::SplittableText((s1, s2))) => {
                    result.tokens.push(TextToken::SplittableText((s1, s2)));
                }
                Ok(Token::ParentFilter(filter)) => {
                    result.filters.parents.insert(filter);
                }
                Ok(Token::UsageFilter(filter)) => {
                    result.filters.usages.insert(filter);
                }
                Ok(Token::TypeFilter(filter)) => {
                    result.filters.types.insert(filter);
                }
                Ok(Token::LocationSort(location)) => {
                    result.sorting.location.insert(location);
                }
                Err(_) => {
                    warn!("Error in query parsing");
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parent_filter() {
        for filter in ["in:", "@"] {
            for sep in ["", " "] {
                assert_eq!(
                    ParsedQuery::from(format!("{filter}{sep}foo").as_str()).filters,
                    Filter {
                        parents: HashSet::from(["foo".to_string()]),
                        ..Default::default()
                    }
                );
            }
        }
    }

    #[test]
    fn usage_filters() {
        for filter in ["usage:", "nutzung:", "="] {
            for sep in ["", " "] {
                assert_eq!(
                    ParsedQuery::from(format!("{filter}{sep}foo").as_str()).filters,
                    Filter {
                        usages: HashSet::from(["foo".to_string()]),
                        ..Default::default()
                    }
                );
            }
        }
    }

    #[test]
    fn type_filters() {
        for sep in ["", " "] {
            assert_eq!(
                ParsedQuery::from(format!("type:{sep}foo").as_str()).filters,
                Filter {
                    types: HashSet::from(["foo".to_string()]),
                    ..Default::default()
                }
            );
        }
    }

    #[test]
    fn location_sort() {
        for sep in ["", " "] {
            assert_eq!(
                ParsedQuery::from(format!("near:{sep}45.32,59.3").as_str()).sorting,
                Sorting {
                    location: HashSet::from(["45.32,59.3".to_string()]),
                }
            );
        }
        for sep in ["", " "] {
            assert_eq!(
                ParsedQuery::from(format!("near:{sep}45.3,59.00000003").as_str()).sorting,
                Sorting {
                    location: HashSet::from(["45.3,59.00000003".to_string()]),
                }
            );
        }
    }

    #[test]
    fn text_token() {
        assert_eq!(
            ParsedQuery::from("foo").tokens,
            vec![TextToken::Text("foo".to_string())]
        );
        assert_eq!(
            ParsedQuery::from("foo foo").tokens,
            vec![
                TextToken::Text("foo".to_string()),
                TextToken::Text("foo".to_string()),
            ]
        );
        assert_eq!(
            ParsedQuery::from("foo hs1").tokens,
            vec![
                TextToken::Text("foo".to_string()),
                TextToken::SplittableText(("hs".to_string(), "1".to_string())),
            ]
        );
    }

    #[test]
    fn text_filter_mixed() {
        assert_eq!(
            ParsedQuery::from("foo in:abc bar @abc foo near:45.32,59.3").tokens,
            vec![
                TextToken::Text("foo".to_string()),
                TextToken::Text("bar".to_string()),
                TextToken::Text("foo".to_string()),
            ]
        );
        assert_eq!(
            ParsedQuery::from(
                "foo in:abc bar @abc =def usage:dd nutzung:gh type:fdh foo near:45.32,59.3"
            )
            .tokens,
            vec![
                TextToken::Text("foo".to_string()),
                TextToken::Text("bar".to_string()),
                TextToken::Text("foo".to_string()),
            ]
        );
    }
}
