use logos::{Lexer, Logos};

/// An irregular split is defined as at least a letter followed by at least one number.
/// Treating words like MW1801 or CH22206 differently has improvements in relevancy for room-level searches.
fn irregular_split(lex: &mut Lexer<Token>) -> (String, String) {
    let slice = lex.slice();
    let mut split = slice.len();
    for (i, c) in slice.char_indices().rev() {
        if c.is_ascii_digit() {
            split = i;
        } else {
            break;
        }
    }
    let (text, numbers) = slice.split_at(split);
    (text.to_string(), numbers.to_string())
}

/// Parses the query string into a list of tokens
/// priority between tokens is set as follows
/// 1. quoted `Text`
/// 2. `SplittableText`
/// 3. `Text`
/// 4. skip
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[regex("\"[^\"]+\"", | lex | lex.slice().trim_matches('"').to_string(), priority = 3)]
    #[regex(r"[^ \t\n\f]+", | lex | lex.slice().to_string(), priority = 1)]
    Text(String),

    #[regex("[a-zA-Z]+[0-9]+", irregular_split, priority = 2)]
    SplittableText((String, String)),
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        assert_eq!(Token::lexer("").next(), None);
        assert_eq!(Token::lexer("\t").next(), None);
        assert_eq!(Token::lexer("\n").next(), None);
        assert_eq!(Token::lexer("  ").next(), None);
    }

    #[test]
    fn quoting() {
        let mut lexer = Token::lexer("\"");
        assert_eq!(lexer.next(), Some(Ok(Token::Text(String::from("\"")))));
        assert_eq!(lexer.next(), None);

        let mut lexer = Token::lexer("\"\"");
        assert_eq!(lexer.next(), Some(Ok(Token::Text(String::from("\"\"")))));
        assert_eq!(lexer.next(), None);

        let mut lexer = Token::lexer("\" \"\"");
        assert_eq!(lexer.next(), Some(Ok(Token::Text(" ".to_string()))));
        assert_eq!(lexer.next(), Some(Ok(Token::Text(String::from("\"")))));
        assert_eq!(lexer.next(), None);
        for text in ["a", "a ", "a a ", " a a ", " @ = in: contains: type: a "] {
            let quoted_text = format!("\"{text}\"");
            let mut lexer = Token::lexer(&quoted_text);
            assert_eq!(lexer.next(), Some(Ok(Token::Text(text.to_string()))));
            assert_eq!(lexer.next(), None);
        }
    }

    #[test]
    fn normal_splits() {
        for text in [
            "foo foo",
            "foo\nfoo",
            "foo  foo",
            " foo foo",
            "foo foo ",
            " foo foo ",
        ] {
            let mut lexer = Token::lexer(text);
            assert_eq!(lexer.next(), Some(Ok(Token::Text("foo".to_string()))));
            assert_eq!(lexer.next(), Some(Ok(Token::Text("foo".to_string()))));
            assert_eq!(lexer.next(), None);
        }
    }

    #[test]
    fn irregular_splits() {
        use Token::SplittableText;
        use Token::Text;
        for (text, expected) in [
            (
                "hs1",
                vec![SplittableText(("hs".to_string(), "1".to_string()))],
            ),
            (
                "physik hs1",
                vec![
                    Text("physik".to_string()),
                    SplittableText(("hs".to_string(), "1".to_string())),
                ],
            ),
            (
                "hs1 physik",
                vec![
                    SplittableText(("hs".to_string(), "1".to_string())),
                    Text("physik".to_string()),
                ],
            ),
            (
                "mw1801",
                vec![SplittableText(("mw".to_string(), "1801".to_string()))],
            ),
            (
                "mw180",
                vec![SplittableText(("mw".to_string(), "180".to_string()))],
            ),
            (
                "ch22206",
                vec![SplittableText(("ch".to_string(), "22206".to_string()))],
            ),
            (
                "ch 22206",
                vec![
                    Text("ch".to_string()),
                    Text("22206".to_string()),
                ],
            ),
        ] {
            let mut lexer = Token::lexer(text);
            for token in expected {
                assert_eq!(lexer.next(), Some(Ok(token)));
            }
            assert_eq!(lexer.next(), None);
        }
    }

    #[test]
    fn quoted_irregular_splits() {
        for text in ["hs1", "physik hs1", "hs1 physik", "mw1801", "mw180", "ch22206"] {
            let quoted_text = format!("\"{text}\"");
            let mut lexer = Token::lexer(&quoted_text);
            assert_eq!(lexer.next(), Some(Ok(Token::Text(text.to_string()))));
            assert_eq!(lexer.next(), None);
        }
    }
}
