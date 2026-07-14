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

/// Normalises a room code like `H.003` that was typed with a mis-remembered number
/// of leading zeros (`H.03`, `H.0003`) to the zero-stripped form Meilisearch treats as
/// canonical (`H.3`). Purely numeric tokens are left untouched, because leading zeros
/// are significant in architect names (`00.01.001`) and ids (`5604.00.011`) - stripping
/// them there loses the match.
pub fn strip_room_code_leading_zeros(token: &str) -> String {
    if !token.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return token.to_string();
    }
    let mut out = String::with_capacity(token.len());
    let mut chars = token.chars().peekable();
    while let Some(c) = chars.next() {
        if !c.is_ascii_digit() {
            out.push(c);
            continue;
        }
        let mut run = String::from(c);
        while chars.peek().is_some_and(char::is_ascii_digit) {
            run.push(chars.next().expect("peeked digit is present"));
        }
        // an all-zero run keeps a single digit so the code stays well-formed.
        let trimmed = run.trim_start_matches('0');
        out.push_str(if trimmed.is_empty() { "0" } else { trimmed });
    }
    out
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
    fn room_code_leading_zeros() {
        // letter-prefixed room codes drop insignificant leading zeros per digit run
        assert_eq!(strip_room_code_leading_zeros("H.03"), "H.3");
        assert_eq!(strip_room_code_leading_zeros("H.0003"), "H.3");
        assert_eq!(strip_room_code_leading_zeros("H.003"), "H.3");
        assert_eq!(strip_room_code_leading_zeros("H.3"), "H.3");
        // no leading zeros -> unchanged
        assert_eq!(strip_room_code_leading_zeros("C.3202"), "C.3202");
        assert_eq!(strip_room_code_leading_zeros("N-1406"), "N-1406");
        // an all-zero run collapses to a single zero
        assert_eq!(strip_room_code_leading_zeros("H.00"), "H.0");
        // purely numeric architect names and ids keep their significant zeros
        assert_eq!(strip_room_code_leading_zeros("00.01.001"), "00.01.001");
        assert_eq!(strip_room_code_leading_zeros("5604.00.011"), "5604.00.011");
        assert_eq!(strip_room_code_leading_zeros("0092@5433"), "0092@5433");
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
}
