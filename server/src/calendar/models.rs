use crate::limited::vec::LimitedVec;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub(super) struct CalendarLocation {
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
pub(super) struct LocationEvents {
    pub(super) events: LimitedVec<Event>,
    pub(super) location: CalendarLocation,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub(super) struct Event {
    /// ID of the calendar entry used in TUMonline internally
    #[schema(examples(6424))]
    pub(super) id: i32,
    /// Structured, globaly unique room code
    ///
    /// Included to enable multi-room calendars.
    /// Format: BUILDING.LEVEL.NUMBER
    #[schema(examples("5602.EG.001", "5121.EG.003"))]
    pub(super) room_code: String,
    /// start of the entry
    #[schema(examples("2018-01-01T00:00:00"))]
    pub(super) start_at: DateTime<Utc>,
    /// end of the entry
    #[schema(examples("2019-01-01T00:00:00"))]
    pub(super) end_at: DateTime<Utc>,
    /// German title of the Entry
    #[schema(examples("Quantenteleportation"))]
    pub(super) title_de: String,
    /// English title of the Entry
    #[schema(examples("Quantum teleportation"))]
    pub(super) title_en: String,
    /// Lecture-type
    #[schema(examples("Vorlesung mit Zentralübung"))]
    pub(super) stp_type: Option<String>,
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
    pub(super) entry_type: String,
    /// For some Entrys, we do have more information (what kind of a `lecture` is it? What kind of an other `entry` is it?)
    #[schema(examples("Abhaltung"))]
    pub(super) detailed_entry_type: String,
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
