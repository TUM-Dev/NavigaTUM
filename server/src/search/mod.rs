use std::fmt::{Debug, Formatter};
use std::time::Instant;

use crate::AppData;
use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, web, HttpResponse};
use cached::proc_macro::cached;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use unicode_truncate::UnicodeTruncateStr;

mod search_executor;

#[derive(Deserialize, Debug, Default)]
pub struct SearchQueryArgs {
    q: String,
    limit_buildings: Option<usize>,
    limit_rooms: Option<usize>,
    limit_all: Option<usize>,
    pre_highlight: Option<String>,
    post_highlight: Option<String>,
}

/// Returned search results by this
#[derive(Serialize, Debug)]
pub struct SearchResults {
    sections: Vec<search_executor::ResultsSection>,
    time_ms: u128,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
/// Limit per facet
pub struct Limits {
    pub buildings_count: usize,
    pub rooms_count: usize,
    pub total_count: usize,
}
impl Debug for Limits {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Limits")
            .field("building", &self.buildings_count)
            .field("rooms", &self.rooms_count)
            .field("total", &self.total_count)
            .finish()
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            total_count: 10,
            buildings_count: 5,
            rooms_count: 10,
        }
    }
}

impl From<&SearchQueryArgs> for Limits {
    fn from(args: &SearchQueryArgs) -> Self {
        let total_count = args.limit_all.unwrap_or(10).clamp(0, 1_000);
        Self {
            buildings_count: args
                .limit_buildings
                .unwrap_or(5)
                .clamp(0, 1_000)
                .min(total_count),
            rooms_count: args
                .limit_rooms
                .unwrap_or(10)
                .clamp(0, 1_000)
                .min(total_count),
            total_count,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Highlighting {
    pub pre: String,
    pub post: String,
}
impl Debug for Highlighting {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pre = &self.pre;
        let post = &self.post;
        write!(f, "{pre}..{post}")
    }
}

impl Default for Highlighting {
    fn default() -> Self {
        Self {
            pre: "\u{0019}".to_string(),
            post: "\u{0017}".to_string(),
        }
    }
}
impl From<&SearchQueryArgs> for Highlighting {
    fn from(args: &SearchQueryArgs) -> Self {
        let (pre, post) = (
            args.pre_highlight
                .clone()
                .unwrap_or_else(|| "\u{0019}".to_string()),
            args.post_highlight
                .clone()
                .unwrap_or_else(|| "\u{0017}".to_string()),
        );
        // After 25 char this parameter kind of misses the point it tries to address.
        // for DOS Reasons this is truncated
        let (pre, post) = (
            pre.unicode_truncate(25).0.to_string(),
            post.unicode_truncate(25).0.to_string(),
        );
        Self { pre, post }
    }
}
#[get("/api/search")]
pub async fn search_handler(
    data: web::Data<AppData>,
    web::Query(args): web::Query<SearchQueryArgs>,
) -> HttpResponse {
    let start_time = Instant::now();
    let _ = data.meilisearch_initialised.read().await; // otherwise we could return empty results during initialisation

    let limits = Limits::from(&args);
    let highlighting = Highlighting::from(&args);
    let q = args.q;
    debug!("searching for {q} with a limit of {limits:?} and highlighting of {highlighting:?}");
    let results_sections = cached_geoentry_search(q, highlighting, limits).await;
    debug!("searching returned {results_sections:?}");

    if results_sections.len() != 2 {
        error!(
            "searching returned {len} sections, but expected 2",
            len = results_sections.len()
        );
        return HttpResponse::InternalServerError().body("Internal error");
    }
    let search_results = SearchResults {
        sections: results_sections,
        time_ms: start_time.elapsed().as_millis(),
    };
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::MaxAge(2 * 24 * 60 * 60), // valid for 2d
            CacheDirective::Public,
        ]))
        .json(search_results)
}

// size=1 ~= 0.1Mi
#[cached(size = 200)]
async fn cached_geoentry_search(
    q: String,
    highlighting: Highlighting,
    limits: Limits,
) -> Vec<search_executor::ResultsSection> {
    let ms_url = std::env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
    let client = Client::new(ms_url, std::env::var("MEILI_MASTER_KEY").ok());
    match client {
        Ok(client) => {
            search_executor::do_geoentry_search(&client, q, highlighting, limits)
                .await
                .0
        }
        Err(e) => {
            error!("Cannot connect to meilisearch because {e:?}");
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_limits_high() {
        let input = SearchQueryArgs {
            limit_all: Some(usize::MAX),
            limit_rooms: Some(usize::MAX),
            limit_buildings: Some(usize::MAX),
            ..Default::default()
        };
        let expected = Limits {
            total_count: 1000,
            rooms_count: 1000,
            buildings_count: 1000,
        };
        assert_eq!(Limits::from(&input), expected);
    }

    #[test]
    fn test_limits_low() {
        let input = SearchQueryArgs {
            limit_all: Some(0),
            limit_rooms: Some(0),
            limit_buildings: Some(0),
            ..Default::default()
        };
        let expected = Limits {
            total_count: 0,
            rooms_count: 0,
            buildings_count: 0,
        };
        assert_eq!(Limits::from(&input), expected);
    }

    #[test]
    fn test_limits_default() {
        let input = SearchQueryArgs {
            limit_all: None,
            limit_rooms: None,
            limit_buildings: None,
            ..Default::default()
        };

        let expected = Limits {
            total_count: 10,
            rooms_count: 10,
            buildings_count: 5,
        };
        assert_eq!(Limits::from(&input), expected);
    }

    #[test]
    fn test_limits_implicit() {
        let input = SearchQueryArgs {
            limit_all: Some(10),
            limit_rooms: Some(100),
            limit_buildings: Some(100),
            ..Default::default()
        };
        let expected = Limits {
            total_count: 10,
            rooms_count: 10,
            buildings_count: 10,
        };
        assert_eq!(Limits::from(&input), expected);
    }

    #[test]
    fn test_highlighting_default() {
        let input = SearchQueryArgs::default();
        let expected = Highlighting {
            pre: "\u{19}".into(),
            post: "\u{17}".into(),
        };
        assert_eq!(Highlighting::from(&input), expected);
    }
    #[test]
    fn test_highlighting_empty() {
        let input = SearchQueryArgs {
            pre_highlight: Some("".into()),
            post_highlight: Some("".into()),
            ..Default::default()
        };
        let expected = Highlighting {
            pre: "".into(),
            post: "".into(),
        };
        assert_eq!(Highlighting::from(&input), expected);
    }

    #[test]
    fn test_highlighting_long() {
        let input = SearchQueryArgs {
            pre_highlight: Some("a".repeat(100)),
            post_highlight: Some("z".repeat(100)),
            ..Default::default()
        };
        let expected = Highlighting {
            pre: "a".repeat(25),
            post: "z".repeat(25),
        };
        assert_eq!(Highlighting::from(&input), expected);
    }
    #[test]
    /// Regression test
    /// unicode characters cannot be split
    /// => when we use String::len to split at an index this creates invalid points
    fn test_highlighting_unicode() {
        for i in 0..51 {
            let mut string_with_unsplittable_uinicode = "a".repeat(i);
            string_with_unsplittable_uinicode.push_str(&"ß".repeat(100));
            let input = SearchQueryArgs {
                pre_highlight: Some(string_with_unsplittable_uinicode.clone()),
                post_highlight: Some(string_with_unsplittable_uinicode.clone()),
                ..Default::default()
            };
            let res = Highlighting::from(&input);
            let expected_length: usize = string_with_unsplittable_uinicode
                .chars()
                .take(25)
                .map(|c| c.len_utf8())
                .sum();
            assert_eq!(res.post.len(), expected_length);
            assert_eq!(res.pre.len(), expected_length);
        }
    }
}
