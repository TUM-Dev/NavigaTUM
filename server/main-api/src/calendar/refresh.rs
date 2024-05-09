use std::time::Duration;

use cached::instant::Instant;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::calendar::connectum::APIRequestor;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
struct LocationKey {
    key: String,
}
pub async fn all_entries(pool: &PgPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let mut api = APIRequestor::from(pool);
    loop {
        let start = Instant::now();
        let ids = sqlx::query_as!(LocationKey,r#"
WITH ENTRIES_TO_SCRAPE AS (SELECT KEY,
                                  CASE WHEN last_calendar_scrape_at IS NULL THEN 100 ELSE 1 END          AS priority,
                                  CAST(data -> 'ranking_factors' ->> 'rank_combined' AS INTEGER)         AS rank_combined,
                                  (LAST_CALENDAR_SCRAPE_AT < DATE_SUBTRACT(NOW(), '1 hour'::INTERVAL, 'Europe/Berlin')
                                      OR LAST_CALENDAR_SCRAPE_AT IS NULL)                                AS would_need_scraping,
                                  EXTRACT(EPOCH FROM (NOW() - LAST_CALENDAR_SCRAPE_AT)) / 60             AS minutes_ago,
                                  CALENDAR_URL IS NOT NULL                                               AS can_be_scraped
                           FROM de)

SELECT key
FROM entries_to_scrape
WHERE would_need_scraping AND can_be_scraped
-- priority: has this ever been scraped? => give a good bonus
-- rank_combined: "how important is this room?" (range 1..1k)
-- minutes_ago: "how long since we last scraped it?" (range null,60..)
ORDER BY priority * rank_combined * coalesce(minutes_ago,1) DESC
LIMIT 20"#)
            .fetch_all(pool)
            .await;
        let ids = match ids {
            Ok(ids) => ids,
            Err(e) => {
                error!("Could not download get LocationKeys from the database because {e:?}");
                continue;
            }
        };
        let len = ids.len();
        for LocationKey { key } in ids {
            if let Err(e) = api.refresh(&key).await {
                error!("Could not download calendar for {key} because {e:?}");
            }
        }
        debug!(
            "Downloaded {len} room-calendars took {elapsed:?}",
            elapsed = start.elapsed()
        );
        interval.tick().await;
    }
}
