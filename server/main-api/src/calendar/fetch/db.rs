use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::calendar::fetch::CalendarEntryFetcher;

pub(super) struct DbRequestor {
    pool: PgPool,
    last_calendar_scrape_at: Option<DateTime<Utc>>,
}

impl CalendarEntryFetcher for DbRequestor {
    fn new(pool: &PgPool, last_calendar_scrape_at: &Option<DateTime<Utc>>) -> Self {
        Self {
            pool: pool.clone(),
            last_calendar_scrape_at: *last_calendar_scrape_at,
        }
    }
    async fn fetch(
        &self,
        id: &str,
        start_after: &DateTime<Utc>,
        end_before: &DateTime<Utc>,
    ) -> Result<super::CalendarEntries, crate::BoxedError> {
        let events = sqlx::query_as!(crate::calendar::models::Event, r#"SELECT id,room_code,start_at,end_at,stp_title_de,stp_title_en,stp_type,entry_type AS "entry_type!:crate::calendar::models::EventType",detailed_entry_type
            FROM calendar
            WHERE room_code = $1 AND start_at >= $2 AND end_at <= $3"#,
            id, start_after, end_before)
            .fetch_all(&self.pool)
            .await?;
        let last_scrape = self
            .last_calendar_scrape_at
            .expect("an entry exists in the db, therefore the time of last scrape is known");
        Ok((last_scrape, events))
    }
}
