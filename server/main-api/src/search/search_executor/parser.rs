use crate::search::search_executor::lexer::Token;
use log::warn;
use logos::Logos;
use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    parents: HashSet<String>,
    types: HashSet<String>,
    usages: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextToken {
    Text(String),
    SplittableText((String, String)),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedQuery {
    pub tokens: Vec<TextToken>,
    pub filters: Filter,
}

impl ParsedQuery {
    pub(crate) fn relevant_enough_for_room_highligting(&self) -> bool {
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
                Err(e) => {
                    warn!("Error in query parsing: {e:?}");
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod parser_tests {
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
            ParsedQuery::from("foo in:abc bar @abc foo").tokens,
            vec![
                TextToken::Text("foo".to_string()),
                TextToken::Text("bar".to_string()),
                TextToken::Text("foo".to_string()),
            ]
        );
        assert_eq!(
            ParsedQuery::from("foo in:abc bar @abc =def usage:dd nutzung:gh type:fdh foo").tokens,
            vec![
                TextToken::Text("foo".to_string()),
                TextToken::Text("bar".to_string()),
                TextToken::Text("foo".to_string()),
            ]
        );
    }
}
