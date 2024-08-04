use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::time::sleep;
use tracing::{debug, error};

use crate::calendar::connectum::APIRequestor;
use crate::limited::vec::LimitedVec;

const NUMBER_OF_CONCURRENT_SCRAPES: usize = 3;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
struct LocationKey {
    key: String,
}

#[tracing::instrument(skip(pool))]
async fn entries_which_need_scraping(
    pool: &PgPool,
) -> Result<LimitedVec<LocationKey>, crate::BoxedError> {
    let res = sqlx::query_as!(LocationKey,r#"
WITH ENTRIES_TO_SCRAPE AS (SELECT KEY,
                                  CASE WHEN last_calendar_scrape_at IS NULL THEN 100 ELSE 1 END          AS priority,
                                  CAST(data -> 'ranking_factors' ->> 'rank_combined' AS INTEGER)         AS rank_combined,
                                  (LAST_CALENDAR_SCRAPE_AT < DATE_SUBTRACT(NOW(), '30 minutes'::INTERVAL, 'Europe/Berlin')
                                      OR LAST_CALENDAR_SCRAPE_AT IS NULL)                                AS would_need_scraping,
                                  EXTRACT(EPOCH FROM (NOW() - LAST_CALENDAR_SCRAPE_AT))                  AS seconds_ago,
                                  CALENDAR_URL IS NOT NULL                                               AS can_be_scraped
                           FROM de)

SELECT key
FROM entries_to_scrape
WHERE would_need_scraping AND can_be_scraped
-- priority: has this ever been scraped? => give a good bonus
-- rank_combined: "how important is this room?" (range 1..1k)
-- seconds_ago: "how long since we last scraped it?" (range null,30*60/3=600..)
ORDER BY priority * rank_combined + priority * coalesce(seconds_ago/6,1) DESC
LIMIT 30"#)
        .fetch_all(pool)
        .await?;
    Ok(LimitedVec::from(res))
}

#[tracing::instrument(skip(pool))]
pub async fn all_entries(pool: &PgPool) {
    if let Err(e) = std::env::var("CONNECTUM_OAUTH_CLIENT_ID") {
        error!("Please make sure that CONNECTUM_OAUTH_CLIENT_ID are valid to use calendar features: {e:?}");
        return;
    }
    if let Err(e) = std::env::var("CONNECTUM_OAUTH_CLIENT_SECRET") {
        error!("Please make sure that CONNECTUM_OAUTH_CLIENT_SECRET is valid to use calendar features: {e:?}");
        return;
    }

    let mut api = APIRequestor::from(pool);
    loop {
        let ids = match entries_which_need_scraping(pool).await {
            Ok(ids) => ids,
            Err(e) => {
                error!("Could not download get LocationKeys from the database because {e:?}");
                continue;
            }
        };
        let should_sleep_for_more_results = ids.len() < 20;
        if should_sleep_for_more_results {
            sleep(Duration::from_secs(60)).await;
        }

        request_events(&mut api, ids).await;
    }
}

#[tracing::instrument(skip(api))]
async fn request_events(api: &mut APIRequestor, mut ids: LimitedVec<LocationKey>) {
    debug!("Downloading {len} room-calendars", len = ids.len());
    while let Err(e) = api.try_refresh_token().await {
        error!("retrying to get oauth token because {e:?}");
        sleep(Duration::from_secs(10)).await;
    }
    // we want to scrape all ~2k rooms once per hour
    // 1 thread is 15..20 per minute => we need at least 2 threads
    // this uses a FuturesUnordered which refills itsself to be able to work effectively with lagging tasks
    let mut work_queue = FuturesUnordered::new();
    for _ in 0..NUMBER_OF_CONCURRENT_SCRAPES {
        if let Some(id) = ids.pop() {
            work_queue.push(api.refresh(id.key));
        }
    }

    while let Some(res) = work_queue.next().await {
        if let Err(e) = res {
            error!("Could not download calendar because {e:?}");
        }
        if let Some(id) = ids.pop() {
            work_queue.push(api.refresh(id.key));
        }
    }
}
