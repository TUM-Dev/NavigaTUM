use crate::limited::vec::LimitedVec;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt::{Debug, Display, Formatter};
use tracing::debug;
use tracing::error;
use tracing::warn;

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CalendarLocation {
    /// Structured, globaly unique room code
    ///
    /// Included to enable multi-room calendars.
    /// Format: BUILDING.LEVEL.NUMBER
    #[schema(examples("5602.EG.001", "5121.EG.003"))]
    pub key: String,
    /// name of the entry in a human-readable form
    #[schema(examples(
        "5602.EG.001 (MI HS 1, Friedrich L. Bauer Hörsaal)",
        "5121.EG.003 (Computerraum)"
    ))]
    pub name: String,
    /// last time the calendar was scraped for this room
    #[schema(examples("2039-01-19T03:14:07+01:00", "2042-01-07T00:00:00 UTC"))]
    pub last_calendar_scrape_at: Option<DateTime<Utc>>,
    /// Link to the calendar of the room
    #[schema(examples(
        "https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12543&cReadonly=J",
        "https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=19691&cRes=12559&cReadonly=J"
    ))]
    pub calendar_url: Option<String>,
    /// Type of the entry in a human-readable form
    #[schema(examples("Serverraum", "Büro"))]
    pub type_common_name: String,
    /// type of the entry
    ///
    /// TODO document as a n enum with the following choices:
    /// - `room`
    /// - `building`
    /// - `joined_building`
    /// - `area`
    /// - `site`
    /// - `campus`
    /// - `poi`
    #[schema(examples("room", "building", "joined_building", "area", "site", "campus", "poi"))]
    pub r#type: String,
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

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LocationEvents {
    pub events: LimitedVec<Event>,
    pub location: CalendarLocation,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Event {
    /// ID of the calendar entry used in TUMonline internally
    #[schema(examples(6424))]
    pub id: i32,
    /// Structured, globaly unique room code
    ///
    /// Included to enable multi-room calendars.
    /// Format: BUILDING.LEVEL.NUMBER
    #[schema(examples("5602.EG.001", "5121.EG.003"))]
    pub room_code: String,
    /// start of the entry
    #[schema(examples("2018-01-01T00:00:00"))]
    pub start_at: DateTime<Utc>,
    /// end of the entry
    #[schema(examples("2019-01-01T00:00:00"))]
    pub end_at: DateTime<Utc>,
    /// German title of the Entry
    #[schema(examples("Quantenteleportation"))]
    pub title_de: String,
    /// English title of the Entry
    #[schema(examples("Quantum teleportation"))]
    pub title_en: String,
    /// Lecture-type
    #[schema(examples("Vorlesung mit Zentralübung"))]
    pub stp_type: Option<String>,
    /// What this calendar entry means.
    ///
    /// Each of these should be displayed in a different color
    /// TODO document as an enum with these values via EventType:
    /// - `lecture`
    /// - `exercise`
    /// - `exam`
    /// - `barred`
    /// - `other`
    #[schema(examples("lecture", "exercise", "exam"))]
    pub entry_type: String,
    /// For some Entrys, we do have more information (what kind of a `lecture` is it? What kind of an other `entry` is it?)
    #[schema(examples("Abhaltung"))]
    pub detailed_entry_type: String,
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

impl Event {
    #[tracing::instrument]
    pub async fn store_all(
        pool: &PgPool,
        events: LimitedVec<Event>,
        id: &str,
    ) -> anyhow::Result<()> {
        // insert into db
        let mut tx = pool.begin().await?;
        if let Err(e) = Event::delete_events(&mut tx, id).await {
            error!("could not delete existing events because {e:?}");
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
                "{cnt}/{total} events could not be inserted because of {e:?}",
                total = events.len()
            );
        }
        tx.commit().await?;
        debug!("finished inserting into the db for {id}");
        Ok(())
    }
    #[tracing::instrument(skip(tx))]
    async fn delete_events(
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

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "EventType")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Lecture,
    Exercise,
    Exam,
    Barred,
    Other,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        let _ = str.remove(0);
        let _ = str.remove(str.len() - 1);
        write!(f, "{str}")
    }
}
