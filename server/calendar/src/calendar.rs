use crate::models::Room;
use crate::models::XMLEvent;
use actix_web::{get, web, HttpResponse};
use chrono::NaiveDateTime;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
pub struct QueryArguments {
    start: NaiveDateTime, // eg. 2022-01-01T00:00:00
    end: NaiveDateTime,   // eg. 2022-01-07T00:00:00
}

async fn get_room_information(
    requested_key: &str,
    conn: &PgPool,
) -> Result<Option<(String, NaiveDateTime)>, sqlx::Error> {
    let room = sqlx::query_as!(Room, "SELECT * FROM rooms WHERE key = $1", requested_key)
        .fetch_optional(conn)
        .await?;
    match room {
        Some(r) => {
            let calendar_url = format!(
                "https://campus.tum.de/tumonline/wbKalender.wbRessource?pResNr={id}",
                id = r.tumonline_calendar_id
            );
            Ok(Some((calendar_url, r.last_scrape)))
        }
        None => Ok(None),
    }
}

async fn get_entries(
    requested_key: &str,
    args: &QueryArguments,
    conn: &PgPool,
) -> Result<Vec<XMLEvent>, sqlx::Error> {
    sqlx::query_as!(
        XMLEvent,
        r#"SELECT *
    FROM calendar
    WHERE key = $1 AND dtstart >= $2 AND dtend <= $3
    ORDER BY dtstart"#,
        requested_key,
        args.start,
        args.end
    )
    .fetch_all(conn)
    .await
}

#[get("/api/calendar/{id}")]
pub async fn calendar_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<QueryArguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params.into_inner();
    let results = get_entries(&id, &args, &data.db).await;
    let room_information = get_room_information(&id, &data.db).await;
    match (results, room_information) {
        (Ok(results), Ok(Some((calendar_url, last_room_sync)))) => {
            let last_calendar_sync = results.iter().map(|e| e.last_scrape).min();
            let events = results.into_iter().map(Event::from).collect();
            HttpResponse::Ok().json(Events {
                events,
                last_sync: last_calendar_sync.unwrap_or(last_room_sync),
                calendar_url,
            })
        }
        (_, Ok(None)) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Room not found"),
        (Err(e), _) => {
            error!("Error loading calendar entries: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Error loading calendar")
        }
        (_, Err(e)) => {
            error!("Error loading calendar_url: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Error loading calendar")
        }
    }
}

#[derive(Serialize, Debug)]
struct Events {
    events: Vec<Event>,
    last_sync: NaiveDateTime,
    calendar_url: String,
}

#[derive(Serialize, Debug)]
struct Event {
    id: i32,
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
            "SPERRE" => return (Self::Barred, String::new()),
            "PT" => return (Self::Exam, String::new()),
            "P" => return (Self::Lecture, course_type_name), // Prüfung (geplant) is sometimes used for lectures
            _ => {}
        }
        match xml_event.event_type_id.as_str() {
            "LV" => (Self::Lecture, course_type_name),
            "PT" => (Self::Exam, String::new()),
            "EX" => (Self::Exercise, String::new()),
            _ => match &xml_event.event_type_name {
                Some(event_type_name) => (
                    Self::Other,
                    format!("{}: {}", xml_event.single_event_type_name, event_type_name),
                ),
                None => (Self::Other, xml_event.single_event_type_name.clone()),
            },
        }
    }
}

impl From<XMLEvent> for Event {
    fn from(xml_event: XMLEvent) -> Self {
        let (entry_type, detailed_entry_type) = EventType::from(&xml_event);
        let title = xml_event.event_title;
        Self {
            id: xml_event.single_event_id,
            title,
            start: xml_event.dtstart,
            end: xml_event.dtend,
            entry_type,
            detailed_entry_type,
        }
    }
}
