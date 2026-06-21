use crate::db::calendar::Event;
use crate::external::connectum::APIRequestor;
use crate::limited::vec::LimitedVec;
use futures::StreamExt as _;
use futures::stream::FuturesUnordered;
use prometheus::{IntCounterVec, IntGaugeVec, Opts, Registry};
use serde::{Deserialize, Serialize, Serializer as _};
use sqlx::PgPool;
use std::env;
use std::fmt::{self, Debug, Formatter};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error};

const NUMBER_OF_CONCURRENT_SCRAPES: usize = 3;

/// How often the freshness gauge is recomputed; a cheap aggregate over `de`.
const FRESHNESS_RECOMPUTE_INTERVAL: Duration = Duration::from_mins(1);

/// Outcome of a single calendar scrape, used as the bounded `result` label.
#[derive(Clone, Copy)]
enum ScrapeResult {
    Success,
    FetchError,
    DecodeError,
    StoreError,
}

impl ScrapeResult {
    fn label(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::FetchError => "fetch_error",
            Self::DecodeError => "decode_error",
            Self::StoreError => "store_error",
        }
    }
}

/// Prometheus handles for calendar-scraper health, shared with the scraper tasks.
///
/// The metric vectors are `Arc`-backed, so cloning hands out shared handles.
#[derive(Clone)]
pub struct CalendarMetrics {
    scrape_total: IntCounterVec,
    rooms_by_freshness: IntGaugeVec,
}

impl CalendarMetrics {
    pub fn new(registry: &Registry) -> anyhow::Result<Self> {
        let scrape_total = IntCounterVec::new(
            Opts::new(
                "navigatum_api_calendar_scrape_total",
                "Calendar scrape attempts partitioned by outcome.",
            ),
            &["result"],
        )?;
        let rooms_by_freshness = IntGaugeVec::new(
            Opts::new(
                "navigatum_api_calendar_rooms_by_freshness",
                "Scrapeable rooms bucketed by how long ago their calendar was last scraped.",
            ),
            &["bucket"],
        )?;
        registry.register(Box::new(scrape_total.clone()))?;
        registry.register(Box::new(rooms_by_freshness.clone()))?;
        Ok(Self {
            scrape_total,
            rooms_by_freshness,
        })
    }

    fn record_scrape(&self, result: ScrapeResult) {
        self.scrape_total.with_label_values(&[result.label()]).inc();
    }
}

/// Distribution of scrapeable rooms over freshness buckets, mirroring the
/// scheduler's 60-minute staleness threshold in [`entries_which_need_scraping`].
struct FreshnessBuckets {
    under_60m: i64,
    within_24h: i64,
    over_24h: i64,
    never: i64,
}

#[tracing::instrument(skip(pool))]
async fn freshness_buckets(pool: &PgPool) -> anyhow::Result<FreshnessBuckets> {
    let row = sqlx::query!(
        r#"
SELECT
    COUNT(*) FILTER (WHERE last_calendar_scrape_at >= DATE_SUBTRACT(NOW(), '60 minutes'::INTERVAL, 'Europe/Berlin'))   AS "under_60m!",
    COUNT(*) FILTER (WHERE last_calendar_scrape_at <  DATE_SUBTRACT(NOW(), '60 minutes'::INTERVAL, 'Europe/Berlin')
                      AND  last_calendar_scrape_at >= DATE_SUBTRACT(NOW(), '24 hours'::INTERVAL, 'Europe/Berlin'))     AS "within_24h!",
    COUNT(*) FILTER (WHERE last_calendar_scrape_at <  DATE_SUBTRACT(NOW(), '24 hours'::INTERVAL, 'Europe/Berlin'))     AS "over_24h!",
    COUNT(*) FILTER (WHERE last_calendar_scrape_at IS NULL)                                                            AS "never!"
FROM de
WHERE calendar_url IS NOT NULL"#
    )
    .fetch_one(pool)
    .await?;
    Ok(FreshnessBuckets {
        under_60m: row.under_60m,
        within_24h: row.within_24h,
        over_24h: row.over_24h,
        never: row.never,
    })
}

/// Recompute the freshness gauge on a timer until cancelled; `over_24h` and
/// `never` are the scraper-falling-behind / frozen-rooms signal.
#[tracing::instrument(skip(pool, metrics))]
pub async fn record_freshness(pool: &PgPool, metrics: CalendarMetrics) {
    loop {
        match freshness_buckets(pool).await {
            Ok(b) => {
                let gauge = &metrics.rooms_by_freshness;
                gauge.with_label_values(&["under_60m"]).set(b.under_60m);
                gauge.with_label_values(&["1h_24h"]).set(b.within_24h);
                gauge.with_label_values(&["over_24h"]).set(b.over_24h);
                gauge.with_label_values(&["never"]).set(b.never);
            }
            Err(e) => error!(error = ?e, "could not recompute calendar freshness metrics"),
        }
        sleep(FRESHNESS_RECOMPUTE_INTERVAL).await;
    }
}

#[derive(Serialize, Deserialize, sqlx::Type)]
struct LocationKey {
    key: String,
}

impl Debug for LocationKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.serialize_str(&self.key)
    }
}

#[tracing::instrument(skip(pool))]
async fn entries_which_need_scraping(pool: &PgPool) -> anyhow::Result<LimitedVec<LocationKey>> {
    let res = sqlx::query_as!(LocationKey,r#"
WITH ENTRIES_TO_SCRAPE AS (SELECT KEY,
                                  CASE WHEN last_calendar_scrape_at IS NULL THEN 100 ELSE 1 END          AS boost_if_never_scraped,
                                  CAST(data -> 'ranking_factors' ->> 'rank_combined' AS INTEGER)         AS rank_combined,
                                  (LAST_CALENDAR_SCRAPE_AT < DATE_SUBTRACT(NOW(), '60 minutes'::INTERVAL, 'Europe/Berlin')
                                      OR LAST_CALENDAR_SCRAPE_AT IS NULL)                                AS would_need_scraping,
                                  EXTRACT(EPOCH FROM (NOW() - LAST_CALENDAR_SCRAPE_AT))                  AS seconds_ago,
                                  CALENDAR_URL IS NOT NULL                                               AS can_be_scraped
                           FROM de)

SELECT key
FROM entries_to_scrape
WHERE would_need_scraping AND can_be_scraped
-- boost_if_never_scraped: has this ever been scraped? => give a good bonus
-- rank_combined: "how important is this room?" (range 1..1k)
-- seconds_ago: "how long since we last scraped it?" (range null,30*60/3=600..)
ORDER BY boost_if_never_scraped * rank_combined * coalesce(seconds_ago/6,1) DESC
LIMIT 30"#)
        .fetch_all(pool)
        .await?;
    Ok(LimitedVec::from(res))
}

fn can_never_succeed() -> bool {
    let client_id_invalid = match env::var("CONNECTUM_OAUTH_CLIENT_ID") {
        Err(_) => true,
        Ok(s) => s.trim().is_empty(),
    };
    if client_id_invalid {
        error!(
            "cannot get environment variable CONNECTUM_OAUTH_CLIENT_ID, necessary to refresh all calendars"
        );
        return true;
    }
    let client_secret_invalid = match env::var("CONNECTUM_OAUTH_CLIENT_SECRET") {
        Err(_) => true,
        Ok(s) => s.trim().is_empty(),
    };
    if client_secret_invalid {
        error!(
            "cannot get environment variable CONNECTUM_OAUTH_CLIENT_SECRET, necessary to refresh all calendars"
        );
        return true;
    }
    false
}

#[tracing::instrument(skip(pool, metrics))]
pub async fn all_entries(pool: &PgPool, metrics: CalendarMetrics) {
    if can_never_succeed() {
        return;
    }

    let api = APIRequestor::default();
    loop {
        let ids = match entries_which_need_scraping(pool).await {
            Ok(ids) => ids,
            Err(e) => {
                error!(
                    error = ?e,
                    "Could not download get LocationKeys from the database",
                );
                continue;
            }
        };
        let should_sleep_for_more_results = ids.len() < 20;
        if should_sleep_for_more_results {
            sleep(Duration::from_mins(1)).await;
        }

        refresh_events(pool, &api, &metrics, ids).await;
    }
}

#[tracing::instrument(skip(api, pool, metrics))]
async fn refresh_events(
    pool: &PgPool,
    api: &APIRequestor,
    metrics: &CalendarMetrics,
    mut ids: LimitedVec<LocationKey>,
) {
    debug!(requested_ids_cnt = ids.len(), "downloading room-calendars");
    // we want to scrape all ~2k rooms once per hour
    // 1 thread is 15..20 per minute => we need at least 2 threads
    // this uses a FuturesUnordered which refills itsself to be able to work effectively with lagging tasks
    let mut work_queue = FuturesUnordered::new();
    for _ in 0..NUMBER_OF_CONCURRENT_SCRAPES {
        if let Some(id) = ids.pop() {
            work_queue.push(refresh_single(pool, api.clone(), metrics, id.key));
        }
    }

    while work_queue.next().await.is_some() {
        if let Some(id) = ids.pop() {
            work_queue.push(refresh_single(pool, api.clone(), metrics, id.key));
        }
    }
}

#[tracing::instrument(skip(pool, api, metrics))]
async fn refresh_single(
    pool: &PgPool,
    mut api: APIRequestor,
    metrics: &CalendarMetrics,
    id: String,
) -> anyhow::Result<()> {
    let sync_start = chrono::Utc::now();
    if let Err(e) = Event::update_last_calendar_scrape_at(pool, &id, &sync_start).await {
        error!(error = ?e, "could not update last_calendar_scrape_at");
        return Err(e.into());
    }

    let events = match api.list_events(&id).await {
        Ok(events) => {
            debug!(
                id,
                fetched_events_cnt = events.len(),
                "finished fetching for calendar events",
            );
            events
        }
        Err(e) => {
            // TODO: this measure is to temporarily make the log usefully again until CO accepts my fix
            if e.to_string() == *"error decoding response body" {
                metrics.record_scrape(ScrapeResult::DecodeError);
                debug!(
                    error = "https://gitlab.campusonline.community/tum/connectum/-/issues/118",
                    "Cannot download calendar"
                );
            } else {
                metrics.record_scrape(ScrapeResult::FetchError);
                error!(error = ?e, "Could not download calendar");
            }
            return Err(e);
        }
    };

    let events = events
        .into_iter()
        .map(|mut e| {
            e.room_code.clone_from(&id);
            e
        })
        .map(Event::from)
        .collect::<LimitedVec<_>>();
    if let Err(e) = Event::store_all(pool, events, &id).await {
        metrics.record_scrape(ScrapeResult::StoreError);
        return Err(e);
    }
    metrics.record_scrape(ScrapeResult::Success);
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "tests assert via unwrap on fixtures and infallible metric setup"
    )]
    use super::*;
    use crate::setup::tests::PostgresTestContainer;
    use chrono::{DateTime, Duration as TimeDelta, Utc};

    /// Minimal `de` row payload; `de.calendar_url` is generated from `props.calendar_url`,
    /// so a missing `calendar_url` makes the room unscrapeable.
    fn room_data(key: &str, calendar_url: Option<&str>) -> serde_json::Value {
        let mut props = serde_json::Map::new();
        if let Some(url) = calendar_url {
            props.insert("calendar_url".into(), url.into());
        }
        serde_json::json!({
            "coords": {"lat": 48.1, "lon": 11.5, "source": "inferred"},
            "name": key,
            "type": "room",
            "type_common_name": "Serverraum",
            "props": props,
        })
    }

    async fn insert_room(
        pool: &PgPool,
        key: &str,
        calendar_url: Option<&str>,
        scraped_at: Option<DateTime<Utc>>,
    ) {
        sqlx::query("INSERT INTO de(key, data, last_calendar_scrape_at) VALUES ($1, $2, $3)")
            .bind(key)
            .bind(room_data(key, calendar_url))
            .bind(scraped_at)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn freshness_buckets_partition_scrapeable_rooms() {
        let pg = PostgresTestContainer::new().await;
        let now = Utc::now();
        let url = Some("https://campus.tum.de/x");
        insert_room(
            &pg.pool,
            "fresh.10m",
            url,
            Some(now - TimeDelta::minutes(10)),
        )
        .await;
        insert_room(
            &pg.pool,
            "fresh.30m",
            url,
            Some(now - TimeDelta::minutes(30)),
        )
        .await;
        insert_room(&pg.pool, "mid.3h", url, Some(now - TimeDelta::hours(3))).await;
        insert_room(&pg.pool, "old.30h", url, Some(now - TimeDelta::hours(30))).await;
        insert_room(&pg.pool, "never.scraped", url, None).await;
        // unscrapeable (no calendar_url) => excluded from every bucket.
        insert_room(&pg.pool, "no.url", None, Some(now - TimeDelta::hours(30))).await;

        let b = freshness_buckets(&pg.pool).await.unwrap();
        assert_eq!(b.under_60m, 2);
        assert_eq!(b.within_24h, 1);
        assert_eq!(b.over_24h, 1);
        assert_eq!(b.never, 1);
    }

    #[test]
    fn scrape_outcomes_increment_their_bounded_label() {
        let metrics = CalendarMetrics::new(&Registry::new()).unwrap();
        metrics.record_scrape(ScrapeResult::Success);
        metrics.record_scrape(ScrapeResult::Success);
        metrics.record_scrape(ScrapeResult::DecodeError);

        let total = &metrics.scrape_total;
        assert_eq!(total.with_label_values(&["success"]).get(), 2);
        assert_eq!(total.with_label_values(&["decode_error"]).get(), 1);
        assert_eq!(total.with_label_values(&["fetch_error"]).get(), 0);
        assert_eq!(total.with_label_values(&["store_error"]).get(), 0);
    }

    /// Mirrors the `registry.gather()` text-encode path that `actix-web-prom`
    /// serves on `/api/metrics`, asserting the series surface under their names.
    #[test]
    fn metrics_surface_on_the_shared_registry() {
        use prometheus::{Encoder as _, TextEncoder};

        let registry = Registry::new();
        let metrics = CalendarMetrics::new(&registry).unwrap();
        metrics.record_scrape(ScrapeResult::Success);
        metrics
            .rooms_by_freshness
            .with_label_values(&["never"])
            .set(3);

        let mut buf = Vec::new();
        TextEncoder::new()
            .encode(&registry.gather(), &mut buf)
            .unwrap();
        let text = String::from_utf8(buf).unwrap();

        assert!(text.contains(r#"navigatum_api_calendar_scrape_total{result="success"} 1"#));
        assert!(text.contains(r#"navigatum_api_calendar_rooms_by_freshness{bucket="never"} 3"#));
    }
}
