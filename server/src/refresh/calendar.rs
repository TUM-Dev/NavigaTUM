use crate::db::calendar::Event;
use crate::external::connectum::APIRequestor;
use crate::limited::vec::LimitedVec;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::{Deserialize, Serialize, Serializer};
use sqlx::PgPool;
use std::env;
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error};

const NUMBER_OF_CONCURRENT_SCRAPES: usize = 3;

#[derive(Serialize, Deserialize, sqlx::Type)]
struct LocationKey {
    key: String,
}

impl Debug for LocationKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
        error!("cannot get environment variable CONNECTUM_OAUTH_CLIENT_ID, nessesary to refresh all calendars");
        return true;
    }
    let client_secret_invalid = match env::var("CONNECTUM_OAUTH_CLIENT_SECRET") {
        Err(_) => true,
        Ok(s) => s.trim().is_empty(),
    };
    if client_secret_invalid {
        error!("cannot get environment variable CONNECTUM_OAUTH_CLIENT_SECRET, nessesary to refresh all calendars");
        return true;
    }
    false
}

#[tracing::instrument(skip(pool))]
pub async fn all_entries(pool: &PgPool) {
    if can_never_succeed() {
        return;
    }

    let api = APIRequestor::default();
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

        refresh_events(pool, &api, ids).await;
    }
}

#[tracing::instrument(skip(api))]
async fn refresh_events(pool: &PgPool, api: &APIRequestor, mut ids: LimitedVec<LocationKey>) {
    debug!("Downloading {len} room-calendars", len = ids.len());
    // we want to scrape all ~2k rooms once per hour
    // 1 thread is 15..20 per minute => we need at least 2 threads
    // this uses a FuturesUnordered which refills itsself to be able to work effectively with lagging tasks
    let mut work_queue = FuturesUnordered::new();
    for _ in 0..NUMBER_OF_CONCURRENT_SCRAPES {
        if let Some(id) = ids.pop() {
            work_queue.push(refresh_single(pool, api.clone(), id.key));
        }
    }

    while work_queue.next().await.is_some() {
        if let Some(id) = ids.pop() {
            work_queue.push(refresh_single(pool, api.clone(), id.key));
        }
    }
}

async fn refresh_single(pool: &PgPool, mut api: APIRequestor, id: String) -> anyhow::Result<()> {
    let sync_start = chrono::Utc::now();
    if let Err(e) = Event::update_last_calendar_scrape_at(pool, &id, &sync_start).await {
        error!("could not update last_calendar_scrape_at because {e:?}");
        return Err(e.into());
    }

    let events = match api.list_events(&id).await {
        Ok(events) => {
            debug!(
                "finished fetching for {cnt} calendar events of {id}",
                cnt = events.len(),
            );
            events
        }
        Err(e) => {
            // TODO: this measure is to temporarily make the log usefully again until CO accepts my fix
            if e.to_string() == *"error decoding response body" {
                debug!("Cannot download calendar because of https://gitlab.campusonline.community/tum/connectum/-/issues/118")
            } else {
                error!("Could not download calendar because {e:?}");
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
    Event::store_all(pool, events, &id).await?;
    Ok(())
}
