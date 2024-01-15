use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub(super) struct Events {
    pub(super) events: Vec<Event>,
    pub(super) last_sync: NaiveDateTime,
    pub(super) calendar_url: String,
}

#[derive(Serialize, Debug)]
pub(super) struct Event {
    pub(super) id: i32,
    pub(super) title: String,
    pub(super) start: NaiveDateTime,
    pub(super) end: NaiveDateTime,
    pub(super) entry_type: EventType,
    pub(super) detailed_entry_type: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(super) enum EventType {
    Lecture,
    Exercise,
    Exam,
    Barred,
    Other,
}