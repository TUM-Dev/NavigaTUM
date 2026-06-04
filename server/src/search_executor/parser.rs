use logos::Logos as _;
use std::fmt::{self, Debug, Formatter};
use tracing::warn;

use super::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextToken {
    Text(String),
    SplittableText((String, String)),
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ParsedQuery {
    pub tokens: Vec<TextToken>,
}

impl Debug for ParsedQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedQuery")
            .field("tokens", &self.tokens)
            .finish()
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
                Err(()) => {
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
}
