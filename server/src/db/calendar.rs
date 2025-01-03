use crate::external::connectum::ConnectumEvent;
use crate::limited::hash_map::LimitedHashMap;
use crate::limited::vec::LimitedVec;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use tracing::debug;
use tracing::error;
use tracing::warn;

pub struct CalendarLocation {
    pub key: String,
    pub name: String,
    pub last_calendar_scrape_at: Option<DateTime<Utc>>,
    pub calendar_url: Option<String>,
    pub type_common_name: String,
    pub r#type: String,
}

impl CalendarLocation {
    #[tracing::instrument(skip(pool))]
    pub(crate) async fn get_locations(
        pool: &PgPool,
        ids: &[String],
    ) -> anyhow::Result<LimitedVec<CalendarLocation>> {
        let res = sqlx::query_as!(
        CalendarLocation,
        "SELECT key,name,last_calendar_scrape_at,calendar_url,type,type_common_name FROM de WHERE key = ANY($1::text[])",
        ids
    )
            .fetch_all(pool)
            .await?;
        Ok(LimitedVec(res))
    }
}
impl Debug for CalendarLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_struct("CalendarLocation");
        base.field("building", &self.key).field("name", &self.name);
        if let Some(from) = &self.last_calendar_scrape_at {
            base.field("from", from);
        }
        base.finish()
    }
}

pub struct LocationEvents {
    pub events: LimitedVec<Event>,
    pub location: CalendarLocation,
}
impl LocationEvents {
    #[tracing::instrument(skip(pool))]
    pub(crate) async fn get_from_db(
        pool: &PgPool,
        locations: Vec<CalendarLocation>,
        start_after: &DateTime<Utc>,
        end_before: &DateTime<Utc>,
    ) -> anyhow::Result<LimitedHashMap<String, LocationEvents>> {
        let mut located_events: HashMap<String, LocationEvents> = HashMap::new();
        for location in locations.into_iter() {
            let events = sqlx::query_as!(
            Event,
            r#"SELECT id,room_code,start_at,end_at,title_de,title_en,stp_type,entry_type,detailed_entry_type
            FROM calendar
            WHERE room_code = $1 AND start_at >= $2 AND end_at <= $3"#,
            location.key,
            start_after,
            end_before
        )
                .fetch_all(pool)
                .await?;
            located_events.insert(
                location.key.clone(),
                LocationEvents {
                    location,
                    events: events.into(),
                },
            );
        }
        Ok(LimitedHashMap(located_events))
    }
}

pub struct Event {
    pub id: i32,
    pub room_code: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub title_de: String,
    pub title_en: String,
    pub stp_type: Option<String>,
    pub entry_type: String,
    pub detailed_entry_type: String,
}
impl Event {
    #[tracing::instrument(skip(pool))]
    pub async fn store_all(
        pool: &PgPool,
        events: LimitedVec<Event>,
        id: &str,
    ) -> anyhow::Result<()> {
        // insert into db
        let mut tx = pool.begin().await?;
        if let Err(e) = Event::delete(&mut tx, id).await {
            error!(error = ?e, "could not delete existing events");
            tx.rollback().await?;
            return Err(e.into());
        }
        let mut failed: Option<(usize, sqlx::Error)> = None;
        for event in events.0.iter() {
            // conflicts cannot occur because all values for said room were dropped
            if let Err(e) = event.store(&mut tx).await {
                failed = match failed {
                    Some((i, e0)) => Some((i + 1, e0)),
                    None => Some((1, e)),
                };
            }
        }
        if let Some((cnt, e)) = failed {
            warn!(
                error = ?e,
                cnt,
                total = events.len(),
                "events could not be inserted because",
            );
        }
        tx.commit().await?;
        debug!(?id, "finished inserting into the db");
        Ok(())
    }
    #[tracing::instrument(skip(tx))]
    async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &str,
    ) -> Result<(), sqlx::Error> {
        loop {
            // deliberately somewhat low to not have too long blocking segments
            let res = sqlx::query!(
                r#"
                    WITH rows_to_delete AS (
                        SELECT id
                        FROM calendar WHERE room_code = $1
                        LIMIT 1000
                    )
                    
                    DELETE FROM calendar
                    WHERE id IN (SELECT id FROM rows_to_delete);"#,
                id
            )
            .execute(&mut **tx)
            .await?;
            if res.rows_affected() == 0 {
                return Ok(());
            }
        }
    }
    #[tracing::instrument(skip(pool))]
    pub async fn update_last_calendar_scrape_at(
        pool: &PgPool,
        id: &str,
        scrape_at: &DateTime<Utc>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "UPDATE en SET last_calendar_scrape_at = $1 WHERE key=$2",
            scrape_at,
            id
        )
        .execute(pool)
        .await?;
        sqlx::query!(
            "UPDATE de SET last_calendar_scrape_at = $1 WHERE key=$2",
            scrape_at,
            id
        )
        .execute(pool)
        .await
    }

    #[tracing::instrument(skip(tx))]
    pub async fn store(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO calendar (id,room_code,start_at,end_at,title_de,title_en,stp_type,entry_type,detailed_entry_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
             room_code = EXCLUDED.room_code,
             start_at = EXCLUDED.start_at,
             end_at = EXCLUDED.end_at,
             title_de = EXCLUDED.title_de,
             title_en = EXCLUDED.title_en,
             stp_type = EXCLUDED.stp_type,
             entry_type = EXCLUDED.entry_type,
             detailed_entry_type = EXCLUDED.detailed_entry_type"#,
            self.id,
            self.room_code,
            self.start_at,
            self.end_at,
            self.title_de,
            self.title_en,
            self.stp_type,
            self.entry_type,
            self.detailed_entry_type,
        ).execute(&mut **tx).await
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let duration = (self.end_at - self.start_at).num_minutes();
        f.debug_tuple("Event")
            .field(&format!(
                "{start} ({duration_h}h{duration_min:?}m): {title}",
                start = self.start_at.naive_local(),
                duration_min = duration % 60,
                duration_h = duration / 60,
                title = self.title_de
            ))
            .finish()
    }
}
impl From<ConnectumEvent> for Event {
    fn from(value: ConnectumEvent) -> Self {
        Event {
            id: value.id,
            room_code: value.room_code,
            start_at: value.start_at,
            end_at: value.end_at,
            title_de: value.title_de,
            title_en: value.title_en,
            stp_type: value.stp_type,
            entry_type: value.entry_type,
            detailed_entry_type: value.detailed_entry_type,
        }
    }
}

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(type_name = "EventType")]
#[sqlx(rename_all = "lowercase")]
pub enum EventType {
    Lecture,
    Exercise,
    Exam,
    Barred,
    Other,
}

impl Display for EventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Lecture => write!(f, "lecture"),
            EventType::Exercise => write!(f, "exercise"),
            EventType::Exam => write!(f, "exam"),
            EventType::Barred => write!(f, "barred"),
            EventType::Other => write!(f, "other"),
        }
    }
}
