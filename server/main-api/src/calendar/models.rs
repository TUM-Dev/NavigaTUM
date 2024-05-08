use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Events {
    pub(super) events: Vec<Event>,
    pub(super) last_sync: DateTime<Utc>,
    pub(super) calendar_url: String,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
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
             room_code = $2,
             start_at = $3,
             end_at = $4,
             stp_title_de = $5,
             stp_title_en = $6,
             stp_type = $7,
             entry_type = $8,
             detailed_entry_type = $9"#,
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
