use std::collections::HashMap;
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use log::error;
use serde::Deserialize;
use sqlx::PgPool;

use crate::calendar::models::{CalendarLocation, Event, LocationEvents};

mod connectum;
mod models;
pub mod refresh;

#[derive(Deserialize, Debug)]
pub struct QueryArguments {
    ids: Vec<String>,
    /// eg. 2039-01-19T03:14:07+1
    start_after: DateTime<Utc>,
    /// eg. 2042-01-07T00:00:00 UTC
    end_before: DateTime<Utc>,
}

#[get("/api/calendar")]
pub async fn calendar_handler(
    web::Query(args): web::Query<QueryArguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let ids = args.ids.into_iter().map(|s| s.replace(|c: char| c.is_whitespace() || c.is_control(), "")).collect::<Vec<String>>();
    if ids.len() > 10 {
        return HttpResponse::BadRequest()
            .body("Too many ids to query. We suspect that users don't need this. If you need this limit increased, please send us a message");
    };
    if ids.is_empty() {
        return HttpResponse::BadRequest()
            .body("No id requested");
    };
    let locations = match get_locations(&data.db, &ids).await {
        Ok(l) => l,
        Err(e) => return e
    };
    if let Err(e) = validate_locations(&ids,&locations){
        return e;
    }
    match get_from_db(&data.db, &locations, &args.start_after, &args.end_before).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            error!("could not get entries from the db for {ids:?} because {e:?}");
            HttpResponse::InternalServerError()
                .body("could not get calendar entries, please try again later")
        }
    }
}

fn validate_locations(ids: &[String],locations:&[CalendarLocation])->Result<(),HttpResponse>{
    for id in ids{
        if !locations.iter().any(|l|&l.key==id) {
            return Err(HttpResponse::BadRequest()
                .body("Requested id {id} does not exist"));
        }
    }
    assert_eq!(locations.len(), ids.len());
    for loc in locations {
        if loc.last_calendar_scrape_at.is_none() {
            return Err(HttpResponse::ServiceUnavailable()
                .body(format!("Room {key}/{url:?} calendar entry is currently in the process of being scraped, please try again later", key = loc.key, url = loc.calendar_url)));
        };
    }
    for loc in locations {
        if loc.calendar_url.is_none() {
            return Err(HttpResponse::NotFound()
                .content_type("text/plain")
                .body(format!("Room {key}/{url:?} does not have a calendar", key = loc.key, url = loc.calendar_url)));
        };
    }
    Ok(())
}

async fn get_locations(pool: &PgPool, ids: &[String]) -> Result<Vec<CalendarLocation>, HttpResponse> {
    match sqlx::query_as!(CalendarLocation, "SELECT key,name,last_calendar_scrape_at,calendar_url,type,type_common_name FROM de WHERE key = ANY($1::text[])", ids)
        .fetch_all(pool)
        .await {
        Err(e) => {
            error!("could not refetch due to {e:?}");
            Err(HttpResponse::InternalServerError()
                .body("could not get calendar entries, please try again later"))
        }
        Ok(locations) => Ok(locations),
    }
}

async fn get_from_db(
    pool: &PgPool,
    locations: &[CalendarLocation],
    start_after: &DateTime<Utc>,
    end_before: &DateTime<Utc>,
) -> Result<HashMap<String, LocationEvents>, crate::BoxedError> {
    let mut located_events: HashMap<String, LocationEvents> = HashMap::new();
    for location in locations {
        let events = sqlx::query_as!(Event, r#"SELECT id,room_code,start_at,end_at,stp_title_de,stp_title_en,stp_type,entry_type AS "entry_type!:crate::calendar::models::EventType",detailed_entry_type
            FROM calendar
            WHERE room_code = $1 AND start_at >= $2 AND end_at <= $3"#,
            location.key, start_after, end_before)
            .fetch_all(pool)
            .await?;
        located_events.insert(
            location.key.clone(),
            LocationEvents {
                location: location.clone(),
                events,
            });
    }
    Ok(located_events)
}
