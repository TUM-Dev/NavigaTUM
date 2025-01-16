use logos::{Lexer, Logos};
use regex::Regex;

/// An irregular split is defined as at least a letter and 1-4 numbers
/// Treating words like MW1801 differently has improvements in relevancy for room-level searches
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

/// Removes the specified prefix and additional whitespace from the token
/// e.g. used to remove the "in:" and "@" prefixes from filters
fn remove_prefix(lex: &mut Lexer<Token>, prefix: &'static str) -> String {
    lex.slice()[prefix.len()..].trim().to_string()
}

/// Removes non-ascii characters from the token (replacing them with at most one whitespace)
fn slugify<S: Into<String>>(input: S) -> String {
    let slugify_regex = Regex::new(r"[^a-zA-Z0-9-äöüß.]+").unwrap();
    let slug = slugify_regex
        .replace_all(&input.into(), "-")
        .to_lowercase()
        .replace("--", "-");
    slug.trim_matches('-').to_string()
}

/// Parses the query string into a list of tokens
/// priority between tokens is set as follows
/// 1. Filters (`ParentFilter`,`UsageFilter`,`TypeFilter`) / quoted `Text` / `LocationSort`
/// 2. `SplittableText`
/// 3. `Text`
/// 4. skip
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[regex("\"[^\"]+\"", | lex | lex.slice()[1..lex.slice().len() - 1].to_string(), priority = 3)]
    #[regex(r"[^ \t\n\f]+", | lex | lex.slice().to_string(), priority = 1)]
    Text(String),

    #[regex("[a-zA-Z]+[0-9]{1,4}", irregular_split, priority = 2)]
    SplittableText((String, String)),

    #[regex("in: ?[a-zA-Z0-9-äöüß.]+", | lex | slugify(remove_prefix(lex, "in:")), priority = 3)]
    #[regex("@ ?[a-zA-Z0-9-äöüß.]+", | lex | slugify(remove_prefix(lex, "@")), priority = 3)]
    ParentFilter(String),

    #[regex("near: ?-?[0-9]+[.][0-9.]+,-?[0-9]+[.][0-9.]+", | lex | remove_prefix(lex, "near:"), priority = 3)]
    LocationSort(String), // e.g. near:lat,lon

    #[regex("usage: ?[a-zA-Z0-9-äöüß.]+", | lex | slugify(remove_prefix(lex, "usage:")), priority = 3)]
    #[regex("nutzung: ?[a-zA-Z0-9-äöüß.]+", | lex | slugify(remove_prefix(lex, "nutzung:")), priority = 3)]
    #[regex("= ?[a-zA-Z0-9-äöüß.]+", | lex | slugify(remove_prefix(lex, "=")), priority = 3)]
    UsageFilter(String),

    #[regex("type: ?[a-zA-Z0-9-äöüß.]+", | lex | slugify(remove_prefix(lex, "type:")), priority = 3)]
    #[regex("typ: ?[a-zA-Z0-9-äöüß.]+", | lex | slugify(remove_prefix(lex, "typ:")), priority = 3)]
    TypeFilter(String),
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
    fn test_slugify_identical() {
        // identical
        assert_eq!(&slugify(""), "");
        assert_eq!(&slugify("a"), "a");
        assert_eq!(&slugify("1234567890"), "1234567890");
        assert_eq!(&slugify("äöüßa."), "äöüßa.");
    }

    #[test]
    fn test_slugify() {
        // to-lower
        assert_eq!(&slugify("B"), "b");
        assert_eq!(&slugify("aA"), "aa");
        // leading/tailing "-" get stripped
        assert_eq!(&slugify("-B-"), "b");
        // no double dashes
        assert_eq!(&slugify("a--21"), "a-21");
        assert_eq!(&slugify("a**21"), "a-21");
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
    fn filter_quoting() {
        // filtering and quoting is explicitly not supported.
        // we have not been able to come up with a usecase for complicating things like this
        for text in ["in:", "@", "usage:", "nutzung:", "="] {
            for sep in [" ", ""] {
                for (test_variation, expected_transformation) in
                    [("\"", "\""), ("\"a", "\"a"), ("\"a\"", "a")]
                {
                    let lexed_text = format!("{text}{sep}{test_variation}");
                    let mut lexer = Token::lexer(&lexed_text);
                    if sep.is_empty() {
                        assert_eq!(lexer.next(), Some(Ok(Token::Text(lexed_text.clone()))));
                    } else {
                        assert_eq!(lexer.next(), Some(Ok(Token::Text(text.into()))));
                        assert_eq!(
                            lexer.next(),
                            Some(Ok(Token::Text(expected_transformation.into())))
                        );
                    }
                    assert_eq!(lexer.next(), None);
                }
            }
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
        for text in ["hs1", "physik hs1", "hs1 physik", "mw1801", "mw180"] {
            let quoted_text = format!("\"{text}\"");
            let mut lexer = Token::lexer(&quoted_text);
            assert_eq!(lexer.next(), Some(Ok(Token::Text(text.to_string()))));
            assert_eq!(lexer.next(), None);
        }
    }

    #[test]
    fn filters() {
        let matchings = [
            (vec!["in:", "@"], Token::ParentFilter("foo".to_string())),
            (
                vec!["usage:", "nutzung:", "="],
                Token::UsageFilter("foo".to_string()),
            ),
            (vec!["type:"], Token::TypeFilter("foo".to_string())),
        ];

        for (filters, expected1) in matchings.clone() {
            for filter in filters {
                for sep in ["", " "] {
                    let quoted_text = format!("{filter}{sep}foo");
                    let mut lexer = Token::lexer(&quoted_text);
                    assert_eq!(lexer.next(), Some(Ok(expected1.clone())));
                    assert_eq!(lexer.next(), None);
                }
            }
        }
    }

    #[test]
    fn sortings() {
        for sep in ["", " "] {
            let quoted_text = format!("near:{sep}12.345,6.789");
            let mut lexer = Token::lexer(&quoted_text);
            assert_eq!(
                lexer.next(),
                Some(Ok(Token::LocationSort("12.345,6.789".to_string())))
            );
            assert_eq!(lexer.next(), None);
        }
    }
}
