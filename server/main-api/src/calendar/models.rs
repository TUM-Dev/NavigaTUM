use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Events {
    pub(super) events: Vec<Event>,
    pub(super) last_sync: DateTime<Local>,
    pub(super) calendar_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Event {
    pub(super) id: i32,
    /// e.g. 5121.EG.003
    pub(super) room_code: String,
    /// e.g. 2018-01-01T00:00:00
    pub(super) start_at: DateTime<Local>,
    /// e.g. 2019-01-01T00:00:00
    pub(super) end_at: DateTime<Local>,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(super) enum EventType {
    Lecture,
    Exercise,
    Exam,
    Barred,
    Other,
}