use crate::scraping::tumonline_calendar::XMLEvent;
use crate::utils;
use actix_web::{get, web, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
pub struct CalendarQueryArgs {
    start: NaiveDateTime, // eg. 2022-01-01T00:00:00
    end: NaiveDateTime,   // eg. 2022-01-07T00:00:00
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(calendar_handler);
}

#[get("/{id}")]
pub async fn calendar_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<CalendarQueryArgs>,
) -> HttpResponse {
    let id = params.into_inner();
    let conn = &mut utils::establish_connection();
    use crate::schema::calendar::dsl::*;
    let results = calendar
        .filter(key.eq(&id))
        .filter(dtstart.ge(&args.start))
        .filter(dtend.le(&args.end))
        .load::<XMLEvent>(conn);
    match results {
        Ok(results) => {
            let last_sync = results
                .iter()
                .map(|e| e.last_scrape)
                .max()
                .unwrap_or(Utc::now().naive_utc());
            let events = results.into_iter().map(Event::from).collect();
            HttpResponse::Ok().json(Events { events, last_sync })
        }
        Err(_) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found"),
    }
}

#[derive(Serialize, Debug)]
struct Events {
    events: Vec<Event>,
    last_sync: NaiveDateTime,
}

#[derive(Serialize, Debug)]
struct Event {
    title: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
    entry_type: EventType,
    detailed_entry_type: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum EventType {
    Lecture,
    Exercise,
    Exam,
    Barred,
    Other,
}
impl EventType {
    fn from(xml_event: &XMLEvent) -> (Self, String) {
        // only used for the lecture type
        let course_type_name = xml_event
            .course_type_name
            .clone()
            .unwrap_or_else(|| "Course type is unknown".to_string());
        match xml_event.single_event_type_id.as_str() {
            "SPERRE" => return (EventType::Barred, "".to_string()),
            "PT" => return (EventType::Exam, "".to_string()),
            "P" => return (EventType::Lecture, course_type_name), // Prüfung (geplant) is sometimes used for lectures
            _ => {}
        }
        match xml_event.event_type_id.as_str() {
            "LV" => (EventType::Lecture, course_type_name),
            "PT" => (EventType::Exam, "".to_string()),
            "EX" => (EventType::Exercise, "".to_string()),
            _ => match &xml_event.event_type_name {
                Some(event_type_name) => (
                    EventType::Other,
                    format!("{}: {}", xml_event.single_event_type_name, event_type_name),
                ),
                None => (EventType::Other, xml_event.single_event_type_name.clone()),
            },
        }
    }
}

impl From<XMLEvent> for Event {
    fn from(xml_event: XMLEvent) -> Self {
        let (entry_type, detailed_entry_type) = EventType::from(&xml_event);
        let title = xml_event.event_title;
        Self {
            title,
            start: xml_event.dtstart,
            end: xml_event.dtend,
            entry_type,
            detailed_entry_type,
        }
    }
}
