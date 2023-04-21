use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::Insertable;

#[derive(Insertable, Queryable, AsChangeset)]
#[diesel(table_name = crate::schema::calendar)]
pub struct XMLEvent {
    pub key: String,
    pub tumonline_id: i32,
    pub dtstart: NaiveDateTime,
    pub dtend: NaiveDateTime,
    pub dtstamp: NaiveDateTime,
    pub event_id: i32,
    pub event_title: String,
    pub single_event_id: i32,
    pub single_event_type_id: String,
    pub single_event_type_name: String,
    pub event_type_id: String,
    pub event_type_name: Option<String>,
    pub course_type_name: Option<String>,
    pub course_type: Option<String>,
    pub course_code: Option<String>,
    pub course_semester_hours: Option<i32>,
    pub group_id: Option<String>,
    pub xgroup: Option<String>,
    pub status_id: String,
    pub status: String,
    pub comment: String,
    pub last_scrape: NaiveDateTime,
}
