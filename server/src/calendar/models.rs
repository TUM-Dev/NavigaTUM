use crate::limited::vec::LimitedVec;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct CalendarLocation {
    pub key: String,
    pub name: String,
    pub last_calendar_scrape_at: Option<DateTime<Utc>>,
    pub calendar_url: Option<String>,
    pub type_common_name: String,
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

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct LocationEvents {
    pub(super) events: LimitedVec<Event>,
    pub(super) location: CalendarLocation,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Event {
    pub(super) id: i32,
    /// e.g. 5121.EG.003
    pub(super) room_code: String,
    /// e.g. 2018-01-01T00:00:00
    pub(super) start_at: DateTime<Utc>,
    /// e.g. 2019-01-01T00:00:00
    pub(super) end_at: DateTime<Utc>,
    /// e.g. Quantenteleportation
    pub(super) title_de: String,
    /// e.g. Quantum teleportation
    pub(super) title_en: String,
    /// e.g. Vorlesung mit Zentralübung
    pub(super) stp_type: Option<String>,
    /// e.g. lecture
    /// in reality this is a [EventType]
    pub(super) entry_type: String,
    /// e.g. Abhaltung
    pub(super) detailed_entry_type: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::Type)]
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
