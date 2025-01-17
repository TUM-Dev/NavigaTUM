use std::fmt::{Debug, Formatter};
use std::time::Instant;

use crate::search_executor::{ResultFacet, ResultsSection};
use crate::AppData;
use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, web, HttpResponse};
use cached::proc_macro::cached;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use tokio::join;
use tracing::{debug, error};
use unicode_truncate::UnicodeTruncateStr;

#[derive(Deserialize, Debug, Default, utoipa::IntoParams, utoipa::ToSchema)]
pub struct SearchQueryArgs {
    /// string you want to search for.
    ///
    /// The amounts returned can be controlled using the `limit\*` paramerters.
    ///
    /// The following query-filters are supported:
    /// - `in:<parent>`/`@<parent>`: Only return rooms in the given parent (e.g. `in:5304` or `in:garching`)
    /// - `usage:<type>`/`nutzung:<usage>`/`=<usage>`: Only return entries of the given usage (e.g. `usage:wc` or `usage:büro`)
    /// - `type:<type>`: Only return entries of the given type (e.g. `type:building` or `type:room`)
    /// - `near:<lat>,<lon>`: prioritise sorting the entries by distance to a coordinate
    #[schema(
        min_length = 1,
        examples(
            "mi hs1",
            "sfarching",
            "5606.EG.036",
            "interims",
            "AStA",
            "WC @garching"
        )
    )]
    // TODO ideally, this would be documented as below, but this does for some reaon not work.
    //    examples(
    //    ("mi hs1" = (summary = "\'misspelled\' (according to tumonline) lecture-hall", value = "mi hs1")),
    //    ("sfarching" = (summary = "misspelled campus garching", value = "sfarching")),
    //    ("5606.EG.036" = (summary = "regular room (fsmpic)", value = "5606.EG.036")),
    //    ("interims" = (summary = "\'interims\' Lecture halls", value = "interims")),
    //    ("AStA" = (summary = "common name synonyms for SV", value = "AStA")),
    //))]
    q: String,
    /// Include adresses in the saerch
    ///
    /// Be aware that Nominatim (which we use to do this search) is really slow (~100ms).
    /// Only activate this when you really need it.
    search_addresses: Option<bool>,
    /// Maximum number of buildings/sites to return.
    ///
    /// Clamped to `0`..`1000`.
    /// If this is a problem for you, please open an issue.
    #[schema(default = 5, maximum = 1000, minimum = 0)]
    limit_buildings: Option<usize>,
    /// Maximum number of rooms to return.
    ///
    /// Clamped to `0`..`1000`.
    /// If this is an problem for you, please open an issue.
    #[schema(default = 10, maximum = 1000, minimum = 0)]
    limit_rooms: Option<usize>,
    /// Maximum number of results to return.
    ///
    /// Clamped to `1`..`1000`.
    /// If this is an problem for you, please open an issue.
    #[schema(default = 10, maximum = 1000, minimum = 1)]
    limit_all: Option<usize>,
    /// string to include in front of highlighted sequences.
    ///
    /// If this and `post_highlight` are empty, highlighting is disabled.
    /// For background on the default values, please see [Wikipedia](https://en.wikipedia.org/wiki/C0_and_C1_control_codes#Modified_C0_control_code_sets)).
    #[schema(
        default = "/u0019",
        max_length = 25,
        max_length = 0,
        examples("/u0019", "<em>", "<ais-highlight-00000000>")
    )]
    pre_highlight: Option<String>,
    /// string to include after the highlighted sequences.
    ///
    /// If this and `pre_highlight` are empty, highlighting is disabled.
    /// For background on the default values, please see [Wikipedia](https://en.wikipedia.org/wiki/C0_and_C1_control_codes#Modified_C0_control_code_sets)).
    #[schema(
        default = "/u0017",
        max_length = 25,
        max_length = 0,
        examples("/u0017", "</em>", "</ais-highlight-00000000>")
    )]
    post_highlight: Option<String>,
}

/// Returned search results by this
#[derive(Serialize, utoipa::ToSchema)]
pub struct SearchResponse {
    sections: Vec<ResultsSection>,
    /// Time the search took in the server side, not including network delay
    ///
    /// Maximum as timeout.
    /// other timeouts (browser, your client) may be smaller.
    /// Expected average is `10`..`50` for uncached, regular requests.
    #[schema(example = 8)]
    time_ms: u32,
}

impl Debug for SearchResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_struct("SearchResponse");
        base.field("time_ms", &self.time_ms);
        for section in self.sections.iter() {
            match section.facet {
                ResultFacet::SitesBuildings => {
                    base.field("sites_buildings", section);
                }
                ResultFacet::Rooms => {
                    base.field("rooms", section);
                }
                ResultFacet::Addresses => {
                    base.field("sites_buildings", section);
                }
            }
        }
        base.finish()
    }
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

/// Search entries
///
/// This endpoint is designed to support search-as-you-type results.
///
/// Instead of simply returning a list, the search results are returned in a way to provide a richer experience by splitting them up into sections. You might not necessarily need to implement all types of sections, or all sections features (if you just want to show a list). The order of sections is a suggested order to display them, but you may change this as you like.
///
/// Some fields support highlighting the query terms and it uses \x19 and \x17 to mark the beginning/end of a highlighted sequence.
/// (See [Wikipedia](https://en.wikipedia.org/wiki/C0_and_C1_control_codes#Modified_C0_control_code_sets)).
/// Some text-renderers will ignore them, but in case you do not want to use them, you might want to remove them from the responses via empty `pre_highlight` and `post_highlight` query parameters.
#[utoipa::path(
    tags=["locations"],
    params(SearchQueryArgs),
    responses(
        (status = 200, description = "Search entries", body = SearchResponse, content_type = "application/json"),
        (status = 400, description= "**Bad Request.** Not all fields in the body are present as defined above", body = String, content_type = "text/plain", example = "Query deserialize error: invalid digit found in string"),
        (status = 404, description = "**Not found.** `q` is empty. Since searching for nothing is nonsensical, we dont support this.", body = String, content_type = "text/plain", example = "Not found"),
        (status = 414, description = "**URI Too Long.** The uri you are trying to request is unreasonably long. Search querys dont have thousands of chars..", body = String, content_type = "text/plain"),
    )
)]
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
    let search_addresses = args.search_addresses.unwrap_or(false);
    debug!(q, ?limits, ?highlighting, "quested search");
    let results_sections = cached_geoentry_search(q, highlighting, limits, search_addresses).await;
    debug!(?results_sections, "searching returned");

    if results_sections.len() > 3 {
        error!(
            returned_section_cnt = results_sections.len(),
            "searching did not return expected the amount of sections it expected",
        );
        return HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body("Cannot perform search, please try again later");
    }
    let search_results = SearchResponse {
        sections: results_sections,
        time_ms: start_time.elapsed().as_millis() as u32,
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
    search_addresses: bool,
) -> Vec<ResultsSection> {
    let ms_url = std::env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
    let Ok(client) = Client::new(ms_url, std::env::var("MEILI_MASTER_KEY").ok()) else {
        error!("Failed to create a meilisearch client");
        return if search_addresses {
            crate::search_executor::address_search(&q).await.0
        } else {
            vec![]
        };
    };
    let geoentry_search =
        crate::search_executor::do_geoentry_search(&client, &q, highlighting, limits);
    if search_addresses {
        let address_search = crate::search_executor::address_search(&q);
        let (address_search, mut geoentry_search) = join!(address_search, geoentry_search);
        geoentry_search.0.extend(address_search.0);
        geoentry_search.0
    } else {
        geoentry_search.await.0
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
