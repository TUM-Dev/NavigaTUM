use crate::search::search_executor::lexer::Token;
use log::warn;
use logos::Logos;
use std::collections::HashSet;
use std::ops::Index;

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
    #[deprecated(note = "workaround to keep the current postprocessing working")]
    pub(crate) fn len(&self) -> usize {
        self.tokens
            .iter()
            .map(|t| match t {
                TextToken::Text(_) => 1,
                TextToken::SplittableText(_) => 2,
            })
            .sum()
    }
}

impl Index<usize> for ParsedQuery {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len());
        let mut current_index = 0;
        for item in self.tokens.iter() {
            match item {
                TextToken::Text(t) => {
                    if current_index == index {
                        return t;
                    }
                    current_index += 1;
                }
                TextToken::SplittableText((t1, t2)) => {
                    if current_index == index {
                        return t1;
                    }
                    current_index += 1;
                    if current_index == index {
                        return t2;
                    }
                    current_index += 1;
                }
            }
        }
        panic!("index out of bounds")
    }
}

impl From<&str> for ParsedQuery {
    fn from(query: &str) -> Self {
        let mut result = ParsedQuery::default();
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
