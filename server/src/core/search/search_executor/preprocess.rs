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
    pub fn to_query_string(self) -> String {
        let mut s = String::from("");
        for token in self.tokens.iter() {
            if token.closed && !token.quoted {
                s.push_str(&format!("{} ", token.s));
            } else {
                s.push_str(&token.s.clone());
            }
        }

        s
    }
}

#[derive(Debug)]
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

pub(super) fn parse_input_query(q: String) -> SearchInput {
    let input_tokens = tokenize_input_query(q);

    let mut search_tokens = Vec::<SearchToken>::new();
    let mut search_filter = SearchFilter {
        parent: None,
        r#type: None,
        usage: None,
    };
    for token in input_tokens {
        // Quoted tokens are ignored. Note this also marks unclosed tokens at the end search quoted.
        if token.s.starts_with("\"") {
            search_tokens.push(SearchToken {
                s: token.s,
                regular_split: token.regular_split,
                closed: token.closed,
                quoted: true,
            });
        } else {
            // Parse filters
            let mut is_filter = false;
            for prefix in vec!["in:", "@", "usage:", "nutzung:", "=", "type:"] {
                if (&token.s).starts_with(prefix) {
                    is_filter = true;

                    let v = token
                        .s
                        .trim_start_matches(prefix)
                        .trim_start_matches("\"")
                        .trim_end_matches("\"");
                    if v.len() == 0 {
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

pub(super) fn tokenize_input_query(q: String) -> Vec<InputToken> {
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
        if c.is_numeric() && 0 < alphabetic_counter && alphabetic_counter <= 3 {
            tokens.push(InputToken {
                s: q.get(token_start..=(i - 1))
                    .unwrap()
                    .trim_end()
                    .to_lowercase(),
                regular_split: false,
                closed: true,
            });

            token_start = i;
        }

        if (!within_quotes && c.is_whitespace() && i > token_start) ||  // whitespace
           ((within_quotes || !c.is_whitespace()) && i+c.len_utf8() == q.len()) ||  // end of string
           (c == '"')
        {
            // end of quotes
            let raw_token = q.get(token_start..i + c.len_utf8());
            if let Some(token) = raw_token {
                tokens.push(InputToken {
                    // Note: The trim_end also trims within unclosed quotes at the end of the query,
                    //       but currently I don't think this is an issue.
                    s: token.trim_end().to_lowercase(),
                    regular_split: true,
                    // `closed` indicates whether the token has been closed (by whitespace or quote)
                    // at the end, when this is the last token. This is relevant because MeiliSearch
                    // treats whitespace at the end differently, and we might want to imitate that
                    // behaviour.
                    closed: !(i + c.len_utf8() == q.len()
                        && (within_quotes || (c != '"' && !c.is_whitespace()))),
                });
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
