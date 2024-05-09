use std::time::Duration;

use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::calendar::connectum::APIRequestor;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
struct LocationKey {
    key: String,
}
pub async fn entries_hourly(pool: &PgPool) {
    let one_hour = Duration::from_secs(60 * 60);
    let mut interval = tokio::time::interval(one_hour);
    let api = APIRequestor::from(pool);
    loop {
        let ids = sqlx::query_as!(LocationKey,r#"SELECT key
        FROM de
        WHERE calendar_url IS NOT NULL
          AND (last_calendar_scrape_at is NULL OR
               last_calendar_scrape_at < date_subtract(NOW(), '1 hour'::interval, 'Europe/Berlin'))"#)
            .fetch_all(pool)
            .await;
        let ids = match ids {
            Ok(ids) => ids,
            Err(e) => {
                error!("Could not download get LocationKeys from the database because {e:?}");
                continue;
            }
        };
        for LocationKey { key } in ids {
            if let Err(e) = api.refresh(&key).await {
                error!("Could not download calendar for {key} because {e:?}");
            }
        }
        interval.tick().await;
    }
}
