use crate::models::Location;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct EventsCollection {
    pub(super) events: HashMap<String, LocationEvents>,
    pub(super) max_last_sync: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct CalendarLocation {
    pub key: String,
    pub name: String,
    pub last_calendar_scrape_at: Option<DateTime<Utc>>,
    pub calendar_url: Option<String>,
    pub type_common_name: String,
    pub r#type: String,
}

impl From<Location> for CalendarLocation {
    fn from(loc: Location) -> Self {
        Self {
            key: loc.key,
            name: loc.name,
            last_calendar_scrape_at: loc.last_calendar_scrape_at,
            calendar_url: loc.calendar_url,
            type_common_name: loc.type_common_name,
            r#type: loc.r#type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct LocationEvents {
    pub(super) events: Vec<Event>,
    pub(super) location: CalendarLocation,
}

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::Type)]
pub(super) struct Event {
    pub(super) id: i32,
    /// e.g. 5121.EG.003
    pub(super) room_code: String,
    /// e.g. 2018-01-01T00:00:00
    pub(super) start_at: DateTime<Utc>,
    /// e.g. 2019-01-01T00:00:00
    pub(super) end_at: DateTime<Utc>,
    /// e.g. Quantenteleportation
    pub(super) stp_title_de: String,
    /// e.g. Quantum teleportation
    pub(super) stp_title_en: String,
    /// e.g. Vorlesung mit Zentralübung
    pub(super) stp_type: String,
    /// e.g. lecture
    pub(super) entry_type: EventType,
    /// e.g. Abhaltung
    pub(super) detailed_entry_type: String,
}

impl Event {
    pub async fn store(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO calendar (id,room_code,start_at,end_at,stp_title_de,stp_title_en,stp_type,entry_type,detailed_entry_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
             room_code = EXCLUDED.room_code,
             start_at = EXCLUDED.start_at,
             end_at = EXCLUDED.end_at,
             stp_title_de = EXCLUDED.stp_title_de,
             stp_title_en = EXCLUDED.stp_title_en,
             stp_type = EXCLUDED.stp_type,
             entry_type = EXCLUDED.entry_type,
             detailed_entry_type = EXCLUDED.detailed_entry_type"#,
            self.id,
            self.room_code,
            self.start_at,
            self.end_at,
            self.stp_title_de,
            self.stp_title_en,
            self.stp_type,
            self.entry_type.clone() as EventType, // see https://github.com/launchbadge/sqlx/issues/1004 => our type is not possible (?)
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
