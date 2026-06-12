use std::env;
use std::fmt::{self, Debug, Formatter};
use std::time::Instant;

use crate::AppData;
use crate::external::meilisearch::FacetFilter;
use crate::search_executor::{self, ResultFacet, ResultsSection};
use actix_web::dev::Payload;
use actix_web::error::ErrorBadRequest;
use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{FromRequest, HttpRequest, HttpResponse, get, web};
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};
use strum::EnumCount as _;
use tokio::join;
use tracing::{debug, error};
use unicode_truncate::UnicodeTruncateStr as _;

/// Cache key for search results
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SearchCacheKey {
    pub q: String,
    pub limits: Limits,
    pub search_addresses: bool,
    pub formatting_config: FormattingConfig,
    pub filter_in: Vec<String>,
    pub filter_usage: Vec<String>,
    pub filter_type: Vec<FacetFilter>,
    pub near: Option<String>,
}

/// Controls whether long building names inside `parsed_id` are cropped.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Deserialize, Serialize, utoipa::ToSchema, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum CroppingMode {
    /// Crop long names (default, preserves compact UI).
    #[default]
    Crop,
    /// Never crop; always show full names.
    Full,
}

/// Controls how `parsed_id` is built for room results.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Deserialize, Serialize, utoipa::ToSchema, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ParsedIdMode {
    /// Prefer a user-facing prefixed format (default), e.g. `"MW 1801"`.
    #[default]
    Prefixed,
    /// Use raw Roomfinder/arch format, e.g. `"archname@building_id"`.
    Roomfinder,
}

#[derive(Deserialize, Debug, Default, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SearchQueryArgs {
    /// string you want to search for.
    ///
    /// The amounts returned can be controlled using the `limit_*` parameters.
    /// Use `in`, `usage`, `type`, and `near` query parameters for filtering.
    #[schema(
        min_length = 1,
        examples("mi hs1", "sfarching", "5606.EG.036", "interims", "AStA")
    )]
    q: String,

    /// Filter by parent (building, campus, etc.).
    ///
    /// Can be repeated for multiple values (e.g. `&in=garching&in=5304`).
    #[serde(rename = "in", default)]
    #[schema(example = json!(["garching"]))]
    filter_in: Vec<String>,

    /// Filter by usage type (e.g. `wc`, `büro`).
    ///
    /// Can be repeated for multiple values.
    #[serde(default)]
    #[schema(example = json!(["wc"]))]
    usage: Vec<String>,

    /// Filter by facet.
    ///
    /// Can be repeated for multiple values. Unknown values cause a `400`.
    #[serde(rename = "type", default)]
    #[schema(example = json!(["site", "room"]))]
    filter_type: Vec<FacetFilter>,

    /// Sort results by distance to a coordinate (`lat,lon`).
    #[schema(example = "48.123,11.456")]
    near: Option<String>,

    /// Include adresses in the saerch
    ///
    /// Be aware that Nominatim (which we use to do this search) is really slow (~100ms).
    /// Only activate this when you really need it.
    search_addresses: Option<bool>,

    /// Include campus events in the search.
    ///
    /// Most clients don't render events, so this facet is disabled by default.
    /// Requesting `type=event` implies enabling it.
    search_events: Option<bool>,

    /// Maximum number of sites (campus / site / area) to return.
    ///
    /// Clamped to `0`..`1000`.
    /// If this is a problem for you, please open an issue.
    #[schema(default = 5, maximum = 1000, minimum = 0)]
    limit_sites: Option<usize>,

    /// Maximum number of buildings to return.
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

    /// Maximum number of POIs (points of interest) to return.
    ///
    /// Clamped to `0`..`1000`.
    /// If this is a problem for you, please open an issue.
    #[schema(default = 5, maximum = 1000, minimum = 0)]
    limit_pois: Option<usize>,

    /// Maximum number of lectures to return.
    ///
    /// Clamped to `0`..`1000`.
    /// If this is a problem for you, please open an issue.
    #[schema(default = 5, maximum = 1000, minimum = 0)]
    limit_lectures: Option<usize>,

    /// Maximum number of events to return.
    ///
    /// Only has an effect when the event facet is enabled (via `search_events`
    /// or `type=event`).
    /// Clamped to `0`..`1000`.
    /// If this is a problem for you, please open an issue.
    #[schema(default = 5, maximum = 1000, minimum = 0)]
    limit_events: Option<usize>,

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

    /// How to handle cropping of long building names in `parsed_id`.
    ///
    /// - `crop` (default): crop long names (> 25 chars) with an ellipsis.
    /// - `full`: never crop; always show full building names.
    #[serde(default)]
    #[schema(default = "crop", example = "full")]
    #[param(inline)]
    cropping: CroppingMode,

    /// How to format `parsed_id` for rooms.
    ///
    /// - `prefixed` (default): add common building prefixes (e.g. `MW 1801`).
    /// - `roomfinder`: return room codes in Roomfinder format (`archname@building_id`).
    #[serde(default)]
    #[schema(default = "prefixed", example = "roomfinder")]
    #[param(inline)]
    parsed_id: ParsedIdMode,
}

// `web::Query` uses `serde_urlencoded`, which cannot deserialise repeated keys
// (e.g. `?in=garching&in=5304`) into `Vec<String>`. `serde_html_form` does.
impl FromRequest for SearchQueryArgs {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(
            serde_html_form::from_str::<Self>(req.query_string())
                .map_err(|e| ErrorBadRequest(format!("Query deserialize error: {e}"))),
        )
    }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut base = f.debug_struct("SearchResponse");
        base.field("time_ms", &self.time_ms);
        for section in &self.sections {
            match section.facet() {
                ResultFacet::Sites => {
                    base.field("sites", section);
                }
                ResultFacet::Buildings => {
                    base.field("buildings", section);
                }
                ResultFacet::Rooms => {
                    base.field("rooms", section);
                }
                ResultFacet::Pois => {
                    base.field("pois", section);
                }
                ResultFacet::Lectures => {
                    base.field("lectures", section);
                }
                ResultFacet::Events => {
                    base.field("events", section);
                }
                ResultFacet::Addresses => {
                    base.field("addresses", section);
                }
            }
        }
        base.finish()
    }
}

/// Limit per facet
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Limits {
    pub sites_count: usize,
    pub buildings_count: usize,
    pub rooms_count: usize,
    pub pois_count: usize,
    pub lectures_count: usize,
    /// `0` encodes "facet disabled": the event facet is default-disabled, so a
    /// disabled request keeps the federation budget (and thus the result set)
    /// byte-identical to one predating the facet.
    pub events_count: usize,
    pub total_count: usize,
}

impl Limits {
    /// Sum of per-facet caps. Used to size the federation budget upstream.
    #[must_use]
    pub fn per_facet_total(&self) -> usize {
        self.sites_count
            .saturating_add(self.buildings_count)
            .saturating_add(self.rooms_count)
            .saturating_add(self.pois_count)
            .saturating_add(self.lectures_count)
            .saturating_add(self.events_count)
    }
}

impl Debug for Limits {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Limits")
            .field("sites", &self.sites_count)
            .field("buildings", &self.buildings_count)
            .field("rooms", &self.rooms_count)
            .field("pois", &self.pois_count)
            .field("lectures", &self.lectures_count)
            .field("events", &self.events_count)
            .field("total", &self.total_count)
            .finish()
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            total_count: 10,
            sites_count: 5,
            buildings_count: 5,
            rooms_count: 10,
            pois_count: 5,
            lectures_count: 5,
            // Mirrors the parameterless request: the event facet is default-disabled.
            events_count: 0,
        }
    }
}

impl From<&SearchQueryArgs> for Limits {
    fn from(args: &SearchQueryArgs) -> Self {
        let total_count = args.limit_all.unwrap_or(10).clamp(0, 1_000);
        let events_enabled =
            args.search_events.unwrap_or(false) || args.filter_type.contains(&FacetFilter::Event);
        Self {
            sites_count: args
                .limit_sites
                .unwrap_or(5)
                .clamp(0, 1_000)
                .min(total_count),
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
            pois_count: args
                .limit_pois
                .unwrap_or(5)
                .clamp(0, 1_000)
                .min(total_count),
            lectures_count: args
                .limit_lectures
                .unwrap_or(5)
                .clamp(0, 1_000)
                .min(total_count),
            events_count: if events_enabled {
                args.limit_events
                    .unwrap_or(5)
                    .clamp(0, 1_000)
                    .min(total_count)
            } else {
                0
            },
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

/// Configuration options for formatting search results
#[derive(Clone, Default, Eq, PartialEq, Hash)]
pub struct FormattingConfig {
    /// Highlighting configuration
    pub highlighting: Highlighting,
    /// How `parsed_id` should be cropped.
    pub cropping: CroppingMode,
    /// How `parsed_id` should be formatted for rooms.
    pub parsed_id: ParsedIdMode,
}

impl Debug for FormattingConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormattingConfig")
            .field("highlighting", &self.highlighting)
            .field("cropping", &self.cropping)
            .field("parsed_id", &self.parsed_id)
            .finish()
    }
}

impl From<&SearchQueryArgs> for FormattingConfig {
    fn from(args: &SearchQueryArgs) -> Self {
        Self {
            highlighting: Highlighting::from(args),
            cropping: args.cropping,
            parsed_id: args.parsed_id,
        }
    }
}

fn slugify(input: &str) -> String {
    let slug = input
        .chars()
        .map(|c| {
            if c.is_alphanumeric()
                || c == '-'
                || c == '_'
                || c == '.'
                || c == 'ä'
                || c == 'ö'
                || c == 'ü'
                || c == 'ß'
            {
                c.to_lowercase().next().unwrap_or(c)
            } else {
                '-'
            }
        })
        .collect::<String>()
        .replace("--", "-");
    slug.trim_matches('-').to_string()
}

fn build_meilisearch_filter(
    filter_in: &[String],
    usage: &[String],
    filter_type: &[FacetFilter],
) -> String {
    let mut filters = vec![];
    if !filter_in.is_empty() {
        let parents: Vec<String> = filter_in.iter().map(|s| slugify(s)).collect();
        let parents_debug: Vec<&str> = parents.iter().map(String::as_str).collect();
        filters.push(format!(
            "((parent_keywords IN {parents_debug:?}) OR (parent_building_names IN {parents_debug:?}) OR (campus IN {parents_debug:?}))"
        ));
    }
    if !filter_type.is_empty() {
        // Allowlist enforced by `serde` at deserialise time - unknown values
        // already turned into a 400 before we got here.
        let facets: Vec<&str> = filter_type.iter().map(|f| f.as_str()).collect();
        filters.push(format!("(facet IN {facets:?})"));
    }
    if !usage.is_empty() {
        let usages: Vec<String> = usage.iter().map(|s| slugify(s)).collect();
        let usages_debug: Vec<&str> = usages.iter().map(String::as_str).collect();
        filters.push(format!("(usage IN {usages_debug:?})"));
    }
    filters.join(" AND ")
}

fn build_meilisearch_sorting(near: Option<&String>) -> Vec<String> {
    match near {
        Some(loc) => vec![format!("_geoPoint({loc}):asc")],
        None => vec![],
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
#[get("/api/search", wrap = "actix_middleware_etag::Etag::default()")]
pub async fn search_handler(data: web::Data<AppData>, args: SearchQueryArgs) -> HttpResponse {
    if args.q.len() > 1000 {
        return HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("The query is too long");
    }
    let start_time = Instant::now();
    // Block until initialisation has finished: the maintenance task holds the write lock until
    // meilisearch is populated, so acquiring (and immediately releasing) the read lock here keeps
    // us from returning empty results during initialisation.
    drop(data.meilisearch_initialised.read().await);

    let limits = Limits::from(&args);
    let formatting_config = FormattingConfig::from(&args);
    let q = args.q;
    let search_addresses = args.search_addresses.unwrap_or(false);
    let filter_in = args.filter_in;
    let filter_usage = args.usage;
    let filter_type = args.filter_type;
    let near = args.near;

    debug!(q, ?limits, ?formatting_config, "requested search");

    let cache_key = SearchCacheKey {
        q: q.clone(),
        limits: limits.clone(),
        search_addresses,
        formatting_config: formatting_config.clone(),
        filter_in: filter_in.clone(),
        filter_usage: filter_usage.clone(),
        filter_type: filter_type.clone(),
        near: near.clone(),
    };

    let ms_filter = build_meilisearch_filter(&filter_in, &filter_usage, &filter_type);
    let ms_sorting = build_meilisearch_sorting(near.as_ref());

    let results_sections = data
        .search_cache
        .get_with(cache_key, async move {
            do_geoentry_search(
                q,
                limits,
                search_addresses,
                formatting_config,
                ms_filter,
                ms_sorting,
            )
            .await
        })
        .await;

    debug!(?results_sections, "searching returned");

    if results_sections.len() > ResultFacet::COUNT {
        error!(
            returned_section_cnt = results_sections.len(),
            max_section_cnt = ResultFacet::COUNT,
            "searching returned more sections than there are facets",
        );
        return HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body("Cannot perform search, please try again later");
    }

    let search_results = SearchResponse {
        sections: results_sections,
        #[expect(
            clippy::cast_possible_truncation,
            reason = "search latency above ~50 days isn't a useful number to report"
        )]
        time_ms: start_time.elapsed().as_millis() as u32,
    };

    HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::MaxAge(2 * 24 * 60 * 60), // valid for 2d
            CacheDirective::Public,
        ]))
        .json(search_results)
}

async fn do_geoentry_search(
    q: String,
    limits: Limits,
    search_addresses: bool,
    formatting_config: FormattingConfig,
    filter: String,
    sorting: Vec<String>,
) -> Vec<ResultsSection> {
    let ms_url = env::var("MIELI_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
    let Ok(client) = Client::new(ms_url, env::var("MEILI_MASTER_KEY").ok()) else {
        error!("Failed to create a meilisearch client");
        return if search_addresses {
            search_executor::address_search(&q).await.0
        } else {
            vec![]
        };
    };

    let geoentry_search = search_executor::do_geoentry_search(
        &client,
        &q,
        limits,
        formatting_config,
        filter,
        sorting,
    );

    if search_addresses {
        let address_search = search_executor::address_search(&q);
        let (address_search, mut geoentry_search) = join!(address_search, geoentry_search);
        geoentry_search.0.extend(address_search.0);
        geoentry_search.0
    } else {
        geoentry_search.await.0
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        reason = "tests assert via panic/unwrap"
    )]
    use pretty_assertions::assert_eq;

    use super::*;

    fn chars_len(s: &str) -> usize {
        s.chars().count()
    }

    fn assert_limits_invariants(limits: &Limits) {
        assert!(limits.total_count <= 1_000);
        assert!(limits.sites_count <= 1_000);
        assert!(limits.buildings_count <= 1_000);
        assert!(limits.rooms_count <= 1_000);
        assert!(limits.pois_count <= 1_000);
        assert!(limits.lectures_count <= 1_000);
        assert!(limits.events_count <= 1_000);

        assert!(limits.sites_count <= limits.total_count);
        assert!(limits.buildings_count <= limits.total_count);
        assert!(limits.rooms_count <= limits.total_count);
        assert!(limits.pois_count <= limits.total_count);
        assert!(limits.lectures_count <= limits.total_count);
        assert!(limits.events_count <= limits.total_count);
    }

    #[test]
    fn limits_default_values_are_sane() {
        let limits = Limits::default();
        assert_eq!(limits.total_count, 10);
        assert_eq!(limits.sites_count, 5);
        assert_eq!(limits.buildings_count, 5);
        assert_eq!(limits.rooms_count, 10);
        assert_eq!(limits.pois_count, 5);
        assert_eq!(limits.lectures_count, 5);
        assert_eq!(limits.events_count, 0);
        assert_limits_invariants(&limits);
    }

    #[test]
    fn limits_are_clamped_to_global_max() {
        let input = SearchQueryArgs {
            limit_all: Some(usize::MAX),
            limit_sites: Some(usize::MAX),
            limit_rooms: Some(usize::MAX),
            limit_buildings: Some(usize::MAX),
            limit_pois: Some(usize::MAX),
            limit_lectures: Some(usize::MAX),
            limit_events: Some(usize::MAX),
            search_events: Some(true),
            ..Default::default()
        };
        let limits = Limits::from(&input);

        assert_eq!(limits.total_count, 1_000);
        assert_eq!(limits.sites_count, 1_000);
        assert_eq!(limits.rooms_count, 1_000);
        assert_eq!(limits.buildings_count, 1_000);
        assert_eq!(limits.pois_count, 1_000);
        assert_eq!(limits.lectures_count, 1_000);
        assert_eq!(limits.events_count, 1_000);
        assert_limits_invariants(&limits);
    }

    #[test]
    fn limits_total_constrains_per_facet_limits() {
        let input = SearchQueryArgs {
            limit_all: Some(10),
            limit_sites: Some(100),
            limit_rooms: Some(100),
            limit_buildings: Some(100),
            limit_pois: Some(100),
            limit_lectures: Some(100),
            limit_events: Some(100),
            search_events: Some(true),
            ..Default::default()
        };
        let limits = Limits::from(&input);

        assert_eq!(limits.total_count, 10);
        assert_eq!(limits.sites_count, 10);
        assert_eq!(limits.rooms_count, 10);
        assert_eq!(limits.buildings_count, 10);
        assert_eq!(limits.pois_count, 10);
        assert_eq!(limits.lectures_count, 10);
        assert_eq!(limits.events_count, 10);
        assert_limits_invariants(&limits);
    }

    #[test]
    fn limits_zero_is_allowed_and_keeps_invariants() {
        let input = SearchQueryArgs {
            limit_all: Some(0),
            limit_sites: Some(0),
            limit_rooms: Some(0),
            limit_buildings: Some(0),
            limit_pois: Some(0),
            limit_lectures: Some(0),
            limit_events: Some(0),
            search_events: Some(true),
            ..Default::default()
        };
        let limits = Limits::from(&input);

        assert_eq!(limits.total_count, 0);
        assert_eq!(limits.sites_count, 0);
        assert_eq!(limits.rooms_count, 0);
        assert_eq!(limits.buildings_count, 0);
        assert_eq!(limits.pois_count, 0);
        assert_eq!(limits.lectures_count, 0);
        assert_eq!(limits.events_count, 0);
        assert_limits_invariants(&limits);
    }

    #[test]
    fn limits_per_facet_total_sums_all_facets() {
        let limits = Limits {
            sites_count: 1,
            buildings_count: 2,
            rooms_count: 3,
            pois_count: 4,
            lectures_count: 5,
            events_count: 6,
            total_count: 100,
        };
        assert_eq!(limits.per_facet_total(), 21);
    }

    #[test]
    fn highlighting_default_is_control_codes() {
        let input = SearchQueryArgs::default();
        let res = Highlighting::from(&input);

        assert_eq!(res.pre, "\u{0019}");
        assert_eq!(res.post, "\u{0017}");
        assert!(chars_len(&res.pre) <= 25);
        assert!(chars_len(&res.post) <= 25);
    }

    #[test]
    fn highlighting_empty_strings_are_preserved() {
        let input = SearchQueryArgs {
            pre_highlight: Some(String::new()),
            post_highlight: Some(String::new()),
            ..Default::default()
        };
        let res = Highlighting::from(&input);

        assert_eq!(res.pre, "");
        assert_eq!(res.post, "");
        assert!(chars_len(&res.pre) <= 25);
        assert!(chars_len(&res.post) <= 25);
    }

    #[test]
    fn highlighting_truncates_at_25_chars_ascii_boundary() {
        let input = SearchQueryArgs {
            pre_highlight: Some("a".repeat(25)),
            post_highlight: Some("z".repeat(26)),
            ..Default::default()
        };
        let res = Highlighting::from(&input);

        assert_eq!(res.pre, "a".repeat(25));
        assert_eq!(res.post, "z".repeat(25));
        assert!(chars_len(&res.pre) <= 25);
        assert!(chars_len(&res.post) <= 25);
    }

    #[test]
    fn highlighting_truncates_by_chars_not_bytes_for_unicode() {
        // Regression test: unicode characters cannot be split
        // => truncation must not create invalid UTF-8 boundaries
        for i in 0..51 {
            let mut s = "a".repeat(i);
            s.push_str(&"ß".repeat(100));

            let input = SearchQueryArgs {
                pre_highlight: Some(s.clone()),
                post_highlight: Some(s.clone()),
                ..Default::default()
            };
            let res = Highlighting::from(&input);

            let expected = s.chars().take(25).collect::<String>();
            assert_eq!(res.pre, expected);
            assert_eq!(res.post, expected);
            assert!(chars_len(&res.pre) <= 25);
            assert!(chars_len(&res.post) <= 25);
        }
    }

    #[test]
    fn formatting_config_uses_highlighting_conversion_and_propagates_modes() {
        let input = SearchQueryArgs {
            pre_highlight: Some("<em>".to_string()),
            post_highlight: Some("</em>".to_string()),
            cropping: CroppingMode::Full,
            parsed_id: ParsedIdMode::Roomfinder,
            ..Default::default()
        };

        let config = FormattingConfig::from(&input);

        // Highlighting should be the same conversion as `Highlighting::from`
        assert_eq!(config.highlighting, Highlighting::from(&input));
        // Modes should be propagated
        assert_eq!(config.cropping, CroppingMode::Full);
        assert_eq!(config.parsed_id, ParsedIdMode::Roomfinder);
    }

    #[test]
    fn filter_empty_params_produce_no_filter() {
        let filter = build_meilisearch_filter(&[], &[], &[]);
        assert_eq!(filter, "");
    }

    #[test]
    fn filter_parent_only() {
        let filter = build_meilisearch_filter(&["garching".to_string()], &[], &[]);
        assert!(filter.contains("parent_keywords"));
        assert!(filter.contains("parent_building_names"));
        assert!(filter.contains("campus"));
        assert!(filter.contains("garching"));
    }

    #[test]
    fn filter_usage_only() {
        let filter = build_meilisearch_filter(&[], &["wc".to_string()], &[]);
        assert!(filter.contains("usage"));
        assert!(filter.contains("wc"));
    }

    #[test]
    fn filter_type_only() {
        let filter = build_meilisearch_filter(&[], &[], &[FacetFilter::Room]);
        assert!(filter.contains("facet"));
        assert!(filter.contains("room"));
    }

    #[test]
    fn filter_combined() {
        let filter = build_meilisearch_filter(
            &["garching".to_string()],
            &["wc".to_string()],
            &[FacetFilter::Room],
        );
        assert!(filter.contains("AND"));
        assert!(filter.contains("garching"));
        assert!(filter.contains("wc"));
        assert!(filter.contains("room"));
    }

    #[test]
    fn filter_slugifies_values() {
        let filter = build_meilisearch_filter(&["Garching".to_string()], &[], &[]);
        assert!(filter.contains("garching"));
        assert!(!filter.contains("Garching"));
    }

    #[test]
    fn filter_type_serializes_all_known_facets() {
        let filter = build_meilisearch_filter(
            &[],
            &[],
            &[
                FacetFilter::Site,
                FacetFilter::Building,
                FacetFilter::Room,
                FacetFilter::Poi,
                FacetFilter::Lecture,
                FacetFilter::Event,
            ],
        );
        insta::assert_snapshot!(filter, @r#"(facet IN ["site", "building", "room", "poi", "lecture", "event"])"#);
    }

    #[test]
    fn query_rejects_unknown_facet_type() {
        // Legacy raw types like `joined_building` and `virtual_room` are no
        // longer valid `?type=` values - serde must reject them so callers get
        // a loud 400 instead of a silently empty filter.
        for bad in ["joined_building", "virtual_room", "campus", "area", ""] {
            let q = format!("q=foo&type={bad}");
            assert!(
                serde_html_form::from_str::<SearchQueryArgs>(&q).is_err(),
                "expected `type={bad}` to be rejected"
            );
        }
    }

    #[test]
    fn sorting_empty_near() {
        let sorting = build_meilisearch_sorting(None);
        assert!(sorting.is_empty());
    }

    #[test]
    fn sorting_with_near() {
        let near = "48.123,11.456".to_string();
        let sorting = build_meilisearch_sorting(Some(&near));
        assert_eq!(sorting, vec!["_geoPoint(48.123,11.456):asc"]);
    }

    #[test]
    fn query_accepts_event_facet_type() {
        let args: SearchQueryArgs = serde_html_form::from_str("q=garnix&type=event").unwrap();
        assert_eq!(args.filter_type, vec![FacetFilter::Event]);
    }

    #[test]
    fn limits_events_are_disabled_by_default() {
        let args: SearchQueryArgs = serde_html_form::from_str("q=garnix").unwrap();
        let limits = Limits::from(&args);
        assert_eq!(limits.events_count, 0);
    }

    #[test]
    fn limits_events_enabled_by_search_events_param() {
        let args: SearchQueryArgs =
            serde_html_form::from_str("q=garnix&search_events=true").unwrap();
        let limits = Limits::from(&args);
        assert_eq!(limits.events_count, 5);
    }

    #[test]
    fn limits_event_type_filter_implies_enabling() {
        let args: SearchQueryArgs = serde_html_form::from_str("q=garnix&type=event").unwrap();
        let limits = Limits::from(&args);
        assert_eq!(limits.events_count, 5);
    }

    #[test]
    fn limits_limit_events_only_applies_when_enabled() {
        // Without enabling the facet, the per-facet limit has nothing to limit.
        let args: SearchQueryArgs = serde_html_form::from_str("q=garnix&limit_events=2").unwrap();
        assert_eq!(Limits::from(&args).events_count, 0);

        let args: SearchQueryArgs =
            serde_html_form::from_str("q=garnix&search_events=true&limit_events=2").unwrap();
        assert_eq!(Limits::from(&args).events_count, 2);
    }

    #[test]
    fn query_parses_repeated_filter_keys() {
        let args: SearchQueryArgs =
            serde_html_form::from_str("q=raum&type=room&type=poi&in=garching&in=5304&usage=wc")
                .unwrap();
        assert_eq!(args.q, "raum");
        assert_eq!(args.filter_type, vec![FacetFilter::Room, FacetFilter::Poi]);
        assert_eq!(args.filter_in, vec!["garching", "5304"]);
        assert_eq!(args.usage, vec!["wc"]);
    }

    #[test]
    fn query_parses_without_filters() {
        let args: SearchQueryArgs = serde_html_form::from_str("q=mensa").unwrap();
        assert_eq!(args.q, "mensa");
        assert!(args.filter_in.is_empty());
        assert!(args.filter_type.is_empty());
        assert!(args.usage.is_empty());
        assert!(args.near.is_none());
    }
}
