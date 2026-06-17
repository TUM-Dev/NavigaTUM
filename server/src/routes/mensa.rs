use std::env;
use std::time::Duration;

use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{HttpResponse, get, web};
use chrono::{Datelike as _, NaiveDate, Utc};
use moka::future::Cache;
use reqwest::StatusCode;
use reqwest::header::LAST_MODIFIED;
use serde::de::IntoDeserializer as _;
use serde::de::value::Error as DeValueError;
use serde::{Deserialize, Serialize};
use tracing::error;

/// How long a menu may be cached, both downstream (`Cache-Control`) and in-process.
const MENU_MAX_AGE_SECS: u32 = 600;
/// Base of the eat-api feed; per-week JSONs live at `{base}/{canteen}/{year}/{week}.json`.
const DEFAULT_EAT_API_URL: &str = "https://tum-dev.github.io/eat-api";
/// Human-facing menu page linked as the card's "source".
const EAT_API_MENU_PAGE: &str = "https://tum-dev.github.io/eat-api/#!/de";

/// Weekly canteen menu sourced from the TUM-Dev eat-api feed.
///
/// `days` is calendar-ordered and covers the current ISO week plus the next, so a Friday
/// visitor still sees Monday. Closed days are simply absent rather than represented as
/// empty entries; an unknown or fully-closed canteen yields an empty `days` list.
#[derive(Deserialize, Serialize, Clone, Debug, utoipa::ToSchema)]
pub(crate) struct MensaMenuResponse {
    /// Where the menu was sourced from; shown as the "source" link on the card.
    #[schema(examples("https://tum-dev.github.io/eat-api/#!/de/mensa-garching"))]
    source_url: String,
    /// `YYYY-MM-DD` date the feed snapshot was last confirmed (the upstream `Last-Modified`).
    #[schema(examples("2026-06-05"))]
    last_update: String,
    /// Per-day dish lists in calendar order; only days with at least one dish are present.
    days: Vec<MensaMenuDayResponse>,
}

/// One day in a [`MensaMenuResponse`].
#[derive(Deserialize, Serialize, Clone, Debug, utoipa::ToSchema)]
pub(crate) struct MensaMenuDayResponse {
    /// `YYYY-MM-DD` calendar date of the day.
    #[schema(examples("2026-06-10"))]
    date: String,
    /// Dishes served on this day, in the upstream serving order.
    dishes: Vec<MensaMenuDishResponse>,
}

/// One dish on one day of a [`MensaMenuResponse`].
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, utoipa::ToSchema)]
pub(crate) struct MensaMenuDishResponse {
    /// Dish title in the upstream language (German).
    #[schema(examples("Pasta Emiliana mit (Vorder-)Schinken und Erbsen"))]
    name: String,
    /// Short category label upstream uses to group the dish (`Pasta`, `Suppe`, `Studitopf`, ...).
    ///
    /// Omitted when upstream did not classify the dish.
    #[schema(examples("Pasta"))]
    dish_type: Option<String>,
    /// Prices keyed by role. A role is omitted when upstream priced the dish only for some.
    prices: MensaMenuPricesResponse,
    /// Allergen, ingredient, and certification labels for the dish.
    ///
    /// The client maps each code to localized text via its own label dictionary, so labels stay
    /// in sync without a server round-trip on language change.
    labels: Vec<MensaMenuLabel>,
}

/// A single allergen, ingredient, or certification label for a dish.
///
/// Always serialized in our `snake_case` convention (e.g. `gluten`, `chicken_eggs`), regardless
/// of how upstream cased it. The known set mirrors eat-api's `enums/labels.json`; a code not yet
/// in this schema is preserved (lower-cased) as [`MensaMenuLabel::Other`] so the client can still
/// show it instead of the whole menu failing to parse.
#[derive(Serialize, Clone, Debug, PartialEq, Eq, utoipa::ToSchema)]
#[serde(untagged)]
pub(crate) enum MensaMenuLabel {
    Known(MensaMenuLabelKind),
    Other(String),
}

impl<'de> Deserialize<'de> for MensaMenuLabel {
    /// Accepts any casing (eat-api emits `SCREAMING_SNAKE_CASE`, our own output `snake_case`) by
    /// lower-casing before matching, so re-reading our serialized output round-trips losslessly.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let normalized = String::deserialize(deserializer)?.to_ascii_lowercase();
        Ok(
            MensaMenuLabelKind::deserialize(normalized.clone().into_deserializer())
                .map_or_else(|_: DeValueError| Self::Other(normalized), Self::Known),
        )
    }
}

/// The labels eat-api documents in `enums/labels.json`, normalized to `snake_case` in our output.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MensaMenuLabelKind {
    Gluten,
    Wheat,
    Rye,
    Barley,
    Oat,
    Spelt,
    Hybrids,
    Shellfish,
    ChickenEggs,
    Fish,
    Peanuts,
    Soy,
    Milk,
    Lactose,
    Almonds,
    Hazelnuts,
    Walnuts,
    Cashews,
    Pecan,
    Pistachios,
    Macadamia,
    Celery,
    Mustard,
    Sesame,
    Sulphurs,
    Sulfites,
    Lupin,
    Molluscs,
    ShellFruits,
    Bavaria,
    Msc,
    Dyestuff,
    Preservatives,
    Antioxidants,
    FlavorEnhancer,
    Waxed,
    Phosphates,
    Sweeteners,
    Phenylalanine,
    CocoaContainingGrease,
    Gelatin,
    Alcohol,
    Pork,
    Beef,
    Veal,
    WildMeat,
    Lamb,
    Garlic,
    Poultry,
    Cereal,
    Meat,
    Vegan,
    Vegetarian,
}

/// Per-role price block for a [`MensaMenuDishResponse`].
///
/// Each field is `None` when upstream did not price the dish for that role.
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, utoipa::ToSchema)]
pub(crate) struct MensaMenuPricesResponse {
    students: Option<MensaMenuPriceResponse>,
    staff: Option<MensaMenuPriceResponse>,
    guests: Option<MensaMenuPriceResponse>,
}

/// One role's price for a dish.
///
/// `price_per_unit` and `unit` are upstream-optional because flat-rate dishes (e.g. a fixed
/// `1.00 €` Studitopf) carry only a `base_price`.
#[serde_with::skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, utoipa::ToSchema)]
pub(crate) struct MensaMenuPriceResponse {
    /// Flat amount in Euros charged before any unit upcharge.
    #[schema(examples(1.0))]
    base_price: f64,
    /// Additional amount in Euros charged per `unit` (e.g. per 100g).
    #[schema(examples(0.9))]
    price_per_unit: Option<f64>,
    /// Unit the `price_per_unit` is charged against (e.g. `100g`).
    #[schema(examples("100g"))]
    unit: Option<String>,
}

/// Upstream weekly payload. Only `days` is consumed; any extra fields are ignored.
#[derive(Deserialize)]
struct EatApiWeek {
    #[serde(default)]
    days: Vec<MensaMenuDayResponse>,
}

/// Outcome of fetching a single ISO week from eat-api.
enum WeekOutcome {
    /// Upstream published no menu for this week (a bare `404`); a common "closed" case.
    Absent,
    /// Upstream returned a menu, with its `Last-Modified` date when present.
    Present {
        days: Vec<MensaMenuDayResponse>,
        last_modified: Option<NaiveDate>,
    },
}

/// Read-through proxy for the eat-api weekly canteen feeds.
///
/// Holds a shared [`reqwest::Client`] and a short-lived TTL cache so repeated requests for the
/// same canteen stay polite to upstream. Cloning shares both the connection pool and the cache,
/// so every worker sees one cache.
#[derive(Clone)]
pub struct EatApiMenus {
    client: reqwest::Client,
    base_url: String,
    cache: Cache<String, MensaMenuResponse>,
}

impl Default for EatApiMenus {
    fn default() -> Self {
        Self::new(env::var("EAT_API_URL").unwrap_or_else(|_| DEFAULT_EAT_API_URL.to_string()))
    }
}

impl EatApiMenus {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .gzip(true)
            .build()
            .expect("the eat-api request client builder is correctly configured");
        Self {
            client,
            base_url,
            cache: Cache::builder()
                .max_capacity(128)
                .time_to_live(Duration::from_secs(u64::from(MENU_MAX_AGE_SECS)))
                .build(),
        }
    }

    /// Return the current+next-ISO-week menu for `canteen`, fetching from eat-api on a cache miss.
    ///
    /// `today` is injected rather than read from the clock so the ISO-week arithmetic is testable.
    /// Returns an error only on a non-404 upstream failure; both-weeks-404 yields an empty menu.
    async fn menu(&self, canteen: &str, today: NaiveDate) -> anyhow::Result<MensaMenuResponse> {
        if let Some(cached) = self.cache.get(canteen).await {
            return Ok(cached);
        }

        let [this_week, next_week] = iso_weeks(today);
        let (current, upcoming) = tokio::join!(
            self.fetch_week(canteen, this_week),
            self.fetch_week(canteen, next_week),
        );

        let mut days = Vec::new();
        let mut last_modified: Option<NaiveDate> = None;
        for outcome in [current?, upcoming?] {
            if let WeekOutcome::Present {
                days: week_days,
                last_modified: week_last_modified,
            } = outcome
            {
                days.extend(week_days);
                last_modified = last_modified.max(week_last_modified);
            }
        }

        let response = MensaMenuResponse {
            source_url: format!("{EAT_API_MENU_PAGE}/{canteen}"),
            last_update: last_modified
                .unwrap_or(today)
                .format("%Y-%m-%d")
                .to_string(),
            days,
        };
        self.cache
            .insert(canteen.to_string(), response.clone())
            .await;
        Ok(response)
    }

    /// Fetch one ISO week. A `404` maps to [`WeekOutcome::Absent`]; any other failure is an error.
    async fn fetch_week(
        &self,
        canteen: &str,
        (year, week): (i32, u32),
    ) -> anyhow::Result<WeekOutcome> {
        let url = format!(
            "{base}/{canteen}/{year}/{week:02}.json",
            base = self.base_url
        );
        let response = self.client.get(&url).send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(WeekOutcome::Absent);
        }
        let response = response.error_for_status()?;
        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .and_then(parse_http_date);
        let week: EatApiWeek = response.json().await?;
        Ok(WeekOutcome::Present {
            days: week.days,
            last_modified,
        })
    }
}

/// `true` iff `slug` matches `^[a-z0-9-]{1,40}$`, the shape eat-api canteen ids take.
fn is_valid_canteen(slug: &str) -> bool {
    (1..=40).contains(&slug.len())
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// The ISO `(year, week)` pairs for `today`'s week and the next, covering a weekend look-ahead.
fn iso_weeks(today: NaiveDate) -> [(i32, u32); 2] {
    [iso_pair(today), iso_pair(today + chrono::Duration::days(7))]
}

fn iso_pair(date: NaiveDate) -> (i32, u32) {
    let iso = date.iso_week();
    (iso.year(), iso.week())
}

/// Parse an HTTP-date (`Last-Modified`) into its calendar date, ignoring the time of day.
fn parse_http_date(raw: &str) -> Option<NaiveDate> {
    chrono::DateTime::parse_from_rfc2822(raw)
        .ok()
        .map(|dt| dt.date_naive())
}

#[derive(Deserialize, utoipa::IntoParams)]
struct MensaPathParams {
    /// eat-api canteen slug.
    #[param(example = "mensa-garching")]
    canteen: String,
}

/// Get a canteen's live menu
///
/// Proxies the TUM-Dev eat-api feed for the current and next ISO week at request time, so the
/// menu stays fresh without a rebuild. The German dish `labels` are mapped to localized text by
/// the client.
///
/// An unknown or closed canteen returns `200` with an empty `days` list; only an actual upstream
/// outage returns `502`, letting callers tell "closed" from "broken".
#[utoipa::path(
    tags=["mensa"],
    params(MensaPathParams),
    responses(
        (status = 200, description = "**Menu** for the **canteen**, current and next ISO week merged", body = MensaMenuResponse, content_type = "application/json"),
        (status = 404, description = "**Bad slug.** The canteen id must match `^[a-z0-9-]{1,40}$`", body = String, content_type = "text/plain", example = "Not found"),
        (status = 502, description = "**Upstream failure.** eat-api could not be reached or returned an error", body = String, content_type = "text/plain", example = "eat-api upstream unavailable"),
    )
)]
#[get("/api/mensa/{canteen}")]
pub async fn menu_handler(
    params: web::Path<MensaPathParams>,
    data: web::Data<EatApiMenus>,
) -> HttpResponse {
    let canteen = &params.canteen;
    if !is_valid_canteen(canteen) {
        return HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found");
    }
    match data.menu(canteen, Utc::now().date_naive()).await {
        Ok(menu) => HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::MaxAge(MENU_MAX_AGE_SECS),
                CacheDirective::Public,
            ]))
            .json(menu),
        Err(e) => {
            error!(error = ?e, canteen, "eat-api menu proxy failed");
            HttpResponse::BadGateway()
                .content_type("text/plain")
                .body("eat-api upstream unavailable")
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::clone_on_ref_ptr,
        clippy::absolute_paths,
        clippy::unused_async,
        reason = "tests assert via unwrap, index known-shape fixtures, clone the shared hit counter, reference absolute paths, and need an async signature for the actix mock handler"
    )]
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use actix_web::http::header::CACHE_CONTROL;
    use actix_web::test::{TestRequest, call_service, init_service};
    use actix_web::{App, web};
    use chrono::Weekday;
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};

    use super::*;

    #[test]
    fn canteen_slug_validation() {
        assert!(is_valid_canteen("mensa-garching"));
        assert!(is_valid_canteen("stubistro-arcisstr"));
        assert!(is_valid_canteen("a"));
        assert!(is_valid_canteen(&"a".repeat(40)));

        assert!(!is_valid_canteen(""));
        assert!(!is_valid_canteen(&"a".repeat(41)));
        assert!(!is_valid_canteen("Mensa-Garching"));
        assert!(!is_valid_canteen("mensa_garching"));
        assert!(!is_valid_canteen("mensa garching"));
        assert!(!is_valid_canteen("../etc/passwd"));
    }

    #[test]
    fn known_labels_round_trip_to_upstream_codes() {
        // Mirrors every `enum_name` in webclient/app/data/eat_api_labels.json; guards that the
        // `snake_case` rename matches upstream's lower-cased codes for each known variant.
        const CODES: [&str; 53] = [
            "GLUTEN",
            "WHEAT",
            "RYE",
            "BARLEY",
            "OAT",
            "SPELT",
            "HYBRIDS",
            "SHELLFISH",
            "CHICKEN_EGGS",
            "FISH",
            "PEANUTS",
            "SOY",
            "MILK",
            "LACTOSE",
            "ALMONDS",
            "HAZELNUTS",
            "WALNUTS",
            "CASHEWS",
            "PECAN",
            "PISTACHIOS",
            "MACADAMIA",
            "CELERY",
            "MUSTARD",
            "SESAME",
            "SULPHURS",
            "SULFITES",
            "LUPIN",
            "MOLLUSCS",
            "SHELL_FRUITS",
            "BAVARIA",
            "MSC",
            "DYESTUFF",
            "PRESERVATIVES",
            "ANTIOXIDANTS",
            "FLAVOR_ENHANCER",
            "WAXED",
            "PHOSPHATES",
            "SWEETENERS",
            "PHENYLALANINE",
            "COCOA_CONTAINING_GREASE",
            "GELATIN",
            "ALCOHOL",
            "PORK",
            "BEEF",
            "VEAL",
            "WILD_MEAT",
            "LAMB",
            "GARLIC",
            "POULTRY",
            "CEREAL",
            "MEAT",
            "VEGAN",
            "VEGETARIAN",
        ];
        for code in CODES {
            let label: MensaMenuLabel = serde_json::from_str(&format!("\"{code}\"")).unwrap();
            assert!(
                matches!(label, MensaMenuLabel::Known(_)),
                "{code} did not map to a known label"
            );
            // Output is normalized to our `snake_case` convention regardless of upstream casing.
            let expected = format!("\"{}\"", code.to_ascii_lowercase());
            assert_eq!(
                serde_json::to_string(&label).unwrap(),
                expected,
                "{code} was not snake-cased"
            );
        }

        // Our own `snake_case` output deserializes back to the same variant (round-trips).
        let again: MensaMenuLabel = serde_json::from_str("\"chicken_eggs\"").unwrap();
        assert_eq!(
            again,
            MensaMenuLabel::Known(MensaMenuLabelKind::ChickenEggs)
        );

        // A code added upstream after this release is kept (lower-cased) rather than rejected.
        let unknown: MensaMenuLabel = serde_json::from_str("\"FUTURE_CODE\"").unwrap();
        assert_eq!(unknown, MensaMenuLabel::Other("future_code".to_string()));
    }

    #[test]
    fn menu_response_output_shape() {
        // A representative upstream week: SCREAMING_SNAKE labels (incl. a multi-word and an
        // unrecognized one), a dish with no `dish_type`, and partial per-role prices.
        let upstream = json!({
            "days": [{
                "date": "2026-06-10",
                "dishes": [
                    {
                        "name": "Pasta Emiliana mit (Vorder-)Schinken und Erbsen",
                        "dish_type": "Pasta",
                        "prices": {
                            "students": {"base_price": 1.0, "price_per_unit": 0.9, "unit": "100g"},
                            "staff": {"base_price": 1.9, "price_per_unit": 1.05, "unit": "100g"},
                            "guests": {"base_price": 1.9, "price_per_unit": 1.4, "unit": "100g"}
                        },
                        "labels": ["GLUTEN", "CHICKEN_EGGS", "VEGETARIAN", "SOME_NEW_LABEL"]
                    },
                    {
                        "name": "Apfel",
                        "prices": {"students": {"base_price": 0.5}},
                        "labels": []
                    }
                ]
            }]
        });
        let week: EatApiWeek = serde_json::from_value(upstream).unwrap();
        let response = MensaMenuResponse {
            source_url: format!("{EAT_API_MENU_PAGE}/mensa-garching"),
            last_update: "2026-06-05".to_string(),
            days: week.days,
        };

        // Snapshot the serialized output so the exact wire shape is reviewable on disk.
        insta::assert_json_snapshot!(response);

        // Our serialized output must itself deserialize back into an identical value.
        let serialized = serde_json::to_value(&response).unwrap();
        let roundtrip: MensaMenuResponse = serde_json::from_value(serialized.clone()).unwrap();
        assert_eq!(serde_json::to_value(&roundtrip).unwrap(), serialized);
    }

    #[test]
    fn iso_weeks_spans_the_year_boundary() {
        // 2025-12-29 is ISO week 1 of 2026 already; the prior Sunday is still 2025-W52.
        let sunday = NaiveDate::from_ymd_opt(2025, 12, 28).unwrap();
        assert_eq!(iso_weeks(sunday), [(2025, 52), (2026, 1)]);
    }

    #[test]
    fn http_date_parses_to_calendar_date() {
        assert_eq!(
            parse_http_date("Wed, 21 Oct 2015 07:28:00 GMT"),
            Some(NaiveDate::from_ymd_opt(2015, 10, 21).unwrap())
        );
        assert_eq!(parse_http_date("not a date"), None);
    }

    /// Monday of the given ISO week, used by the mock to make each week's data distinct.
    fn monday_of(year: i32, week: u32) -> NaiveDate {
        NaiveDate::from_isoywd_opt(year, week, Weekday::Mon).unwrap()
    }

    fn week_body(year: i32, week: u32) -> Value {
        json!({
            "days": [{
                "date": monday_of(year, week).format("%Y-%m-%d").to_string(),
                "dishes": [{
                    "name": "Test Dish",
                    "prices": {"students": {"base_price": 1.0}},
                    "labels": ["GLUTEN"],
                }],
            }],
        })
    }

    fn last_modified_header(year: i32, week: u32) -> String {
        monday_of(year, week)
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string()
    }

    async fn mock_week(
        path: web::Path<(String, i32, u32)>,
        hits: web::Data<Arc<AtomicUsize>>,
    ) -> HttpResponse {
        let (canteen, year, week) = path.into_inner();
        hits.fetch_add(1, Ordering::SeqCst);
        match canteen.as_str() {
            "closed" => HttpResponse::NotFound().finish(),
            "broken" => HttpResponse::InternalServerError().finish(),
            "ok-no-last-modified" => HttpResponse::Ok().json(week_body(year, week)),
            _ => HttpResponse::Ok()
                .insert_header(("Last-Modified", last_modified_header(year, week)))
                .json(week_body(year, week)),
        }
    }

    /// Spawn an in-process eat-api stand-in; returns its base URL and a request counter.
    async fn start_mock() -> (String, Arc<AtomicUsize>) {
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_data = web::Data::new(hits.clone());
        let server = actix_web::HttpServer::new(move || {
            App::new()
                .app_data(hits_data.clone())
                .route("/{canteen}/{year}/{week}.json", web::get().to(mock_week))
        })
        .workers(1)
        .bind(("127.0.0.1", 0))
        .unwrap();
        let addr = server.addrs()[0];
        actix_web::rt::spawn(server.run());
        (format!("http://{addr}"), hits)
    }

    #[actix_web::test]
    async fn merges_both_weeks_and_takes_latest_last_modified() {
        let (base_url, hits) = start_mock().await;
        let api = EatApiMenus::new(base_url);
        let today = NaiveDate::from_ymd_opt(2026, 6, 17).unwrap();
        let [_, next_week] = iso_weeks(today);

        let menu = api.menu("mensa-garching", today).await.unwrap();

        assert_eq!(hits.load(Ordering::SeqCst), 2);
        assert_eq!(menu.days.len(), 2, "one day from each fetched week");
        assert_ne!(menu.days[0].date, menu.days[1].date);
        // `last_update` is the max of the two `Last-Modified` dates, i.e. the upcoming week's.
        assert_eq!(
            menu.last_update,
            monday_of(next_week.0, next_week.1)
                .format("%Y-%m-%d")
                .to_string()
        );
        assert_eq!(
            menu.source_url,
            format!("{EAT_API_MENU_PAGE}/mensa-garching")
        );
    }

    #[actix_web::test]
    async fn both_weeks_absent_yields_empty_menu_dated_today() {
        let (base_url, _hits) = start_mock().await;
        let api = EatApiMenus::new(base_url);
        let today = NaiveDate::from_ymd_opt(2026, 6, 17).unwrap();

        let menu = api.menu("closed", today).await.unwrap();

        assert!(menu.days.is_empty());
        assert_eq!(menu.last_update, "2026-06-17");
    }

    #[actix_web::test]
    async fn missing_last_modified_falls_back_to_today() {
        let (base_url, _hits) = start_mock().await;
        let api = EatApiMenus::new(base_url);
        let today = NaiveDate::from_ymd_opt(2026, 6, 17).unwrap();

        let menu = api.menu("ok-no-last-modified", today).await.unwrap();

        assert!(!menu.days.is_empty());
        assert_eq!(menu.last_update, "2026-06-17");
    }

    #[actix_web::test]
    async fn non_404_upstream_failure_is_an_error() {
        let (base_url, _hits) = start_mock().await;
        let api = EatApiMenus::new(base_url);
        let today = NaiveDate::from_ymd_opt(2026, 6, 17).unwrap();

        assert!(api.menu("broken", today).await.is_err());
    }

    #[actix_web::test]
    async fn second_call_is_served_from_cache() {
        let (base_url, hits) = start_mock().await;
        let api = EatApiMenus::new(base_url);
        let today = NaiveDate::from_ymd_opt(2026, 6, 17).unwrap();

        api.menu("mensa-garching", today).await.unwrap();
        api.menu("mensa-garching", today).await.unwrap();

        assert_eq!(
            hits.load(Ordering::SeqCst),
            2,
            "the second call hits no upstream"
        );
    }

    #[actix_web::test]
    async fn invalid_slug_is_404_without_an_upstream_call() {
        let (base_url, hits) = start_mock().await;
        let app = init_service(
            App::new()
                .app_data(web::Data::new(EatApiMenus::new(base_url)))
                .service(menu_handler),
        )
        .await;

        for slug in ["Mensa-Garching", "mensa_garching", &"a".repeat(41)] {
            let req = TestRequest::get()
                .uri(&format!("/api/mensa/{slug}"))
                .to_request();
            let resp = call_service(&app, req).await;
            assert_eq!(resp.status(), 404);
        }
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    #[actix_web::test]
    async fn valid_request_sets_cache_control() {
        let (base_url, _hits) = start_mock().await;
        let app = init_service(
            App::new()
                .app_data(web::Data::new(EatApiMenus::new(base_url)))
                .service(menu_handler),
        )
        .await;

        let req = TestRequest::get()
            .uri("/api/mensa/mensa-garching")
            .to_request();
        let resp = call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let cache_control = resp.headers().get(CACHE_CONTROL).unwrap().to_str().unwrap();
        assert!(
            cache_control.contains("max-age=600"),
            "got {cache_control:?}"
        );
    }

    #[actix_web::test]
    async fn upstream_failure_maps_to_502() {
        let (base_url, _hits) = start_mock().await;
        let app = init_service(
            App::new()
                .app_data(web::Data::new(EatApiMenus::new(base_url)))
                .service(menu_handler),
        )
        .await;

        let req = TestRequest::get().uri("/api/mensa/broken").to_request();
        let resp = call_service(&app, req).await;

        assert_eq!(resp.status(), 502);
    }
}
