#[derive(Debug, Clone)]
struct SearchFilter {
    parent: Option<Vec<String>>,
    r#type: Option<Vec<String>>,
    usage: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub(super) struct SearchInput {
    pub(super) tokens: Vec<SearchToken>,
    #[allow(dead_code)]
    filter: SearchFilter,
}

impl SearchInput {
    pub fn to_query_string(&self) -> String {
        let mut s = String::from("");
        for token in &self.tokens {
            if token.closed && !token.quoted {
                s.push_str(&format!("{} ", token.s));
            } else {
                s.push_str(&token.s.clone());
            }
        }

        s
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(super) struct InputToken {
    s: String,
    regular_split: bool,
    closed: bool,
}

#[derive(Debug, Clone)]
pub(super) struct SearchToken {
    pub(super) s: String,
    #[allow(dead_code)]
    regular_split: bool,
    pub(super) closed: bool,
    pub(super) quoted: bool,
}

pub(super) fn parse_input_query(q: &str) -> SearchInput {
    let input_tokens = tokenize_input_query(q);

    let mut search_tokens = Vec::<SearchToken>::new();
    let mut search_filter = SearchFilter {
        parent: None,
        r#type: None,
        usage: None,
    };
    for token in input_tokens {
        // Quoted tokens are ignored. Note this also marks unclosed tokens at the end search quoted.
        if token.s.starts_with('"') {
            search_tokens.push(SearchToken {
                s: token.s,
                regular_split: token.regular_split,
                closed: token.closed,
                quoted: true,
            });
        } else {
            // Parse filters
            let mut is_filter = false;
            for prefix in ["in:", "@", "usage:", "nutzung:", "=", "type:"] {
                if (&token.s).starts_with(prefix) {
                    is_filter = true;

                    let v = token
                        .s
                        .trim_start_matches(prefix)
                        .trim_start_matches('"')
                        .trim_end_matches('"');
                    if v.is_empty() {
                        // e.g. ' in: ', ' @ ', ' in:"" ' are ignored,
                        continue; // TODO: autosuggest
                    };

                    let filter = match prefix {
                        "in:" | "@" => Some(&mut search_filter.parent),
                        "usage:" | "nutzung:" | "=" => Some(&mut search_filter.usage),
                        "type:" => Some(&mut search_filter.r#type),
                        _ => None,
                    };

                    if let Some(Some(f)) = filter {
                        f.push(v.to_string());
                    } else {
                        *filter.unwrap() = Some(vec![v.to_string()]);
                    }

                    break;
                }
            }

            if !is_filter {
                search_tokens.push(SearchToken {
                    s: token.s,
                    regular_split: token.regular_split,
                    closed: token.closed,
                    quoted: false,
                });
            }
        }
    }

    SearchInput {
        tokens: search_tokens,
        filter: search_filter,
    }
}

pub(super) fn tokenize_input_query(q: &str) -> Vec<InputToken> {
    let mut tokens = Vec::<InputToken>::new();

    // We don't care about unicode here since all split conditions
    // only involve ascii characters.
    let mut within_quotes = false;
    let mut alphabetic_counter = 0;
    let mut token_start = 0;
    for (i, c) in q.char_indices() {
        // Quote escaping is not supported
        if c == '"' {
            within_quotes = !within_quotes;

            // Only closing (even-numbered) quotes do a split (see below).
            // Opening quotes should not split (therefore the continue).
            if within_quotes {
                continue;
            }
        }

        // Note:
        // - Regular splits are splits based on a specific character and technically inclusive
        //   (for quotes at least, whitespace is trimmed),
        // - Irregular splits are splits based on a specific pattern and exclusive.
        //
        // It can happen that two splits need to be made:
        //   "physik hs1" on the last char
        //             ^
        // does an irregular split ("hs") and regular split ("1") because of
        // the end of the query string.
        //
        // For this reason irregular splits are determined before regular splits.

        // There is a special case when up to 3 alphabetic chars are followed by a numeric part.
        // This is intended to split up strings like "MW1250".
        if !within_quotes && c.is_numeric() && 0 < alphabetic_counter && alphabetic_counter <= 3 {
            tokens.push(InputToken {
                s: q.get(token_start..i).unwrap().trim_end().to_lowercase(),
                regular_split: false,
                closed: true,
            });

            token_start = i;
        }

        if (!within_quotes && c.is_whitespace() && i > token_start) ||  // whitespace
           ((within_quotes || !c.is_whitespace()) && i+c.len_utf8() == q.len()) ||  // end of string
           (c == '"')  // end of quotes
        {
            let raw_token = q.get(token_start..i + c.len_utf8());
            if let Some(token) = raw_token {
                if token != "\"\"" {
                    tokens.push(InputToken {
                        s: if within_quotes {
                            // Autoclose quotes for the last token
                            if c != '"' && i+c.len_utf8() == q.len() {
                                format!("{}\"", token.to_lowercase())
                            } else {
                                token.to_lowercase()
                            }
                        } else {
                            token.trim_end().to_lowercase()
                        },
                        regular_split: true,
                        // `closed` indicates whether the token has been closed (by whitespace)
                        // at the end, when this is the last token. This is relevant because MeiliSearch
                        // treats whitespace at the end differently, and we might want to imitate that
                        // behaviour. Quotes are always autoclosed.
                        closed: !(i + c.len_utf8() == q.len() && !c.is_whitespace() && c != '"') || within_quotes,
                    });
                }
            }

            token_start = i + 1;
        } else if !within_quotes && c.is_whitespace() {
            //    ^
            // To avoid empty tokens when there are multiple whitespaces, we need
            // to move the token start even if there was no split.
            token_start = i + 1;
        }

        if c.is_alphabetic() {
            alphabetic_counter += 1;
        } else {
            alphabetic_counter = 0;
        }
    }

    tokens
}

#[cfg(test)]
mod tokenizer_tests {
    use super::*;

    fn reg(s: &str) -> InputToken {
        InputToken {
            s: s.to_string(),
            regular_split: true,
            closed: true,
        }
    }

    fn irreg(s: &str) -> InputToken {
        InputToken {
            s: s.to_string(),
            regular_split: false,
            closed: true,
        }
    }

    fn assert_token(q: String, expected: Vec<InputToken>) {
        assert_eq!(
            tokenize_input_query(q.as_str().clone()),
            expected,
            "tokenization for '{}' failed",
            q
        );
    }

    fn assert_tokens(q: &str, mut expected: Vec<InputToken>) {
        // Variations that end with a space are only tested
        // for strings with closed quotes, because quotes do
        // include spaces at the end into the token
        let unclosed_quotes = q.matches("\"").count() % 2 == 1;
        if !unclosed_quotes {
            let sqs = format!(" {} ", q);
            let qs = format!("{} ", q);
            assert_token(sqs, expected.clone());
            assert_token(qs, expected.clone());
            if !expected.is_empty() {
                let mut last = expected.pop().unwrap();
                // Change `closed` to false for unquoted last token
                last.closed = last.s.ends_with("\"");
                expected.push(last);
            }
        }
        let sq = format!(" {}", q);
        assert_token(sq, expected.clone());
        assert_token(q.to_string(), expected.clone());
    }

    #[macro_export]
    macro_rules! assert_identical {
        ($x:expr) => {
            assert_tokens($x, vec![reg($x)]);
        };
    }

    #[test]
    fn empty() {
        assert_tokens("", vec![]);
        assert_tokens("\t", vec![]);
        assert_tokens("\n", vec![]);
    }

    #[test]
    fn quoting() {
        assert_tokens("\"", vec![]);
        assert_tokens("\"\"", vec![]);
        assert_tokens("\" \"\"", vec![reg("\" \"")]);
        assert_identical!("\"a\"");
        assert_identical!("\"a \"");
        assert_identical!("\"a a \"");
        assert_identical!("\" a a \"");
    }

    #[test]
    fn normal_splits() {
        assert_identical!("foo");
        assert_tokens("foo foo", vec![reg("foo"), reg("foo")]);
        assert_tokens("foo\nfoo", vec![reg("foo"), reg("foo")]);
        assert_tokens("foo   foo", vec![reg("foo"), reg("foo")]);
    }

    #[test]
    fn irregular_splits() {
        assert_tokens("hs1", vec![irreg("hs"), reg("1")]);
        assert_tokens("physik hs1", vec![reg("physik"), irreg("hs"), reg("1")]);
        assert_tokens("hs1 physik", vec![irreg("hs"), reg("1"), reg("physik")]);
        assert_tokens("mw1801", vec![irreg("mw"), reg("1801")]);
        assert_tokens("mw180", vec![irreg("mw"), reg("180")]);
    }

    #[test]
    fn quoted_irregular_splits() {
        assert_identical!("\"hs1\"");
        assert_identical!("\"physik hs1\"");
        assert_identical!("\"hs1 physik\"");
        assert_identical!("\"mw1801\"");
        assert_identical!("\"mw180\"");
    }
}
