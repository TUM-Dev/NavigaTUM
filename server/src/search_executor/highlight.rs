use meilisearch_sdk::search::{MatchRange, SearchResult};

use crate::external::meilisearch::MSHit;

pub(super) struct HighlightContext<'a> {
    pub query: &'a str,
    pub pre: &'a str,
    pub post: &'a str,
}

pub(super) fn highlighted_name_for_hit(
    hit: &SearchResult<MSHit>,
    ctx: &HighlightContext<'_>,
) -> String {
    let raw = hit.result.name().to_string();
    let Some(positions) = hit.matches_position.as_ref() else {
        return raw;
    };
    let Some(name_matches) = positions.get("name") else {
        return raw;
    };
    rebuild_highlighted_name(
        hit.result.name(),
        name_matches,
        ctx.query,
        ctx.pre,
        ctx.post,
    )
}

/// Strip highlight tags from spans that Meilisearch only reached via typo
/// tolerance, so the rendered hit doesn't misrepresent a near-match as exact
/// (issue #513). Spans that are an exact substring of the query or a prefix
/// match retain their tags.
pub(super) fn rebuild_highlighted_name(
    raw_name: &str,
    matches: &[MatchRange],
    query: &str,
    pre: &str,
    post: &str,
) -> String {
    let query_lower = query.to_lowercase();
    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();
    let mut out = String::with_capacity(raw_name.len());
    let mut cursor = 0usize;
    for m in matches {
        if m.indices.as_ref().is_some_and(|i| !i.is_empty()) {
            continue;
        }
        let start = m.start;
        let end = m.start.saturating_add(m.length);
        if start < cursor || end > raw_name.len() {
            continue;
        }
        let Some(span) = raw_name.get(start..end) else {
            continue;
        };
        out.push_str(raw_name.get(cursor..start).unwrap_or_default());
        if span_matches_query(span, &query_lower, &query_tokens) {
            out.push_str(pre);
            out.push_str(span);
            out.push_str(post);
        } else {
            out.push_str(span);
        }
        cursor = end;
    }
    out.push_str(raw_name.get(cursor..).unwrap_or_default());
    out
}

fn span_matches_query(span: &str, query_lower: &str, query_tokens: &[&str]) -> bool {
    // Issue #513 is about numeric/alphanumeric IDs being typo-tolerated and
    // shown as if exact. Spans without digits are almost always legitimate
    // text matches (synonyms like `tb` -> `Bibliothek`, typos like `pyhsik` ->
    // `Physik`, prefix matches like `PH` -> `Physik`) where the highlight is
    // user-helpful even if not strictly verbatim.
    if !span.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }
    let span_lower = span.to_lowercase();
    if query_lower.contains(&span_lower) {
        return true;
    }
    query_tokens
        .iter()
        .any(|tok| !tok.is_empty() && span_lower.starts_with(tok))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn single_exact_match_is_wrapped_in_tags() {
        let raw = "Hauptgebaeude";
        let matches = vec![MatchRange {
            start: 0,
            length: 13,
            indices: None,
        }];
        let out = rebuild_highlighted_name(raw, &matches, "Hauptgebaeude", "<em>", "</em>");
        assert_eq!(out, "<em>Hauptgebaeude</em>");
    }

    #[test]
    fn prefix_match_keeps_full_word_highlight() {
        let raw = "Physik";
        let matches = vec![MatchRange {
            start: 0,
            length: 6,
            indices: None,
        }];
        let out = rebuild_highlighted_name(raw, &matches, "PH 5101 PH5101", "<em>", "</em>");
        assert_eq!(out, "<em>Physik</em>");
    }

    #[test]
    fn alphabetic_synonym_match_keeps_highlight() {
        // `tb` is configured as a synonym for `Bibliothek` in
        // server/src/setup/search_synonyms.yaml. The expanded match is
        // legitimate and the highlight communicates that fact.
        let raw = "Bibliothek";
        let matches = vec![MatchRange {
            start: 0,
            length: 10,
            indices: None,
        }];
        let out = rebuild_highlighted_name(raw, &matches, "tb innenstadt", "<em>", "</em>");
        assert_eq!(out, "<em>Bibliothek</em>");
    }

    #[test]
    fn empty_matches_returns_raw_name() {
        let raw = "Innenhof";
        let out = rebuild_highlighted_name(raw, &[], "garten", "<em>", "</em>");
        assert_eq!(out, "Innenhof");
    }

    #[test]
    fn case_difference_still_counts_as_exact() {
        let raw = "MW1801";
        let matches = vec![MatchRange {
            start: 0,
            length: 2,
            indices: None,
        }];
        let out = rebuild_highlighted_name(raw, &matches, "mw", "<em>", "</em>");
        assert_eq!(out, "<em>MW</em>1801");
    }

    #[test]
    fn mixed_exact_and_typo_spans_preserve_only_exact_tags() {
        let raw = "5511.01.234";
        let matches = vec![
            MatchRange {
                start: 0,
                length: 4,
                indices: None,
            },
            MatchRange {
                start: 5,
                length: 2,
                indices: None,
            },
            MatchRange {
                start: 8,
                length: 3,
                indices: None,
            },
        ];
        let out = rebuild_highlighted_name(raw, &matches, "5510.01.234", "<em>", "</em>");
        assert_eq!(out, "5511.<em>01</em>.<em>234</em>");
    }

    #[test]
    fn typo_match_is_emitted_without_tags() {
        let raw = "5511";
        let matches = vec![MatchRange {
            start: 0,
            length: 4,
            indices: None,
        }];
        let out = rebuild_highlighted_name(raw, &matches, "5510", "<em>", "</em>");
        assert_eq!(out, "5511");
    }
}
