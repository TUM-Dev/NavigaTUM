use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use log::error;
use serde::Deserialize;
use sqlx::PgPool;

use crate::calendar::models::Event;
use crate::models::Location;

mod connectum;
mod models;
pub mod refresh;

#[derive(Deserialize, Debug)]
pub struct QueryArguments {
    /// eg. 2039-01-19T03:14:07+1
    start_after: DateTime<Utc>,
    /// eg. 2042-01-07T00:00:00 UTC
    end_before: DateTime<Utc>,
}

#[get("/api/calendar/{id}")]
pub async fn calendar_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<QueryArguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params
        .into_inner()
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    let location = match get_location(&data.db, &id).await {
        Err(e) => {
            error!("could not refetch due to {e:?}");
            return HttpResponse::InternalServerError()
                .body("could not get calendar entries, please try again later");
        }
        Ok(None) => {
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Room not found");
        }
        Ok(Some(loc)) => loc,
    };
    let Some(last_sync) = location.last_calendar_scrape_at else {
        return HttpResponse::ServiceUnavailable()
          .body("This calendar entry is currently in the process of being scraped, please try again later");
    };
    let Some(calendar_url) = location.calendar_url else {
        return HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Room does not have a calendar");
    };
    match get_from_db(&data.db, &id, &args.start_after, &args.end_before).await {
        Ok(events) => HttpResponse::Ok().json(models::Events {
            events,
            last_sync,
            calendar_url,
        }),
        Err(e) => {
            error!("could not get entries from the db for {id} because {e:?}");
            HttpResponse::InternalServerError()
                .body("could not get calendar entries, please try again later")
        }
    }
}

async fn get_location(pool: &PgPool, id: &str) -> Result<Option<Location>, sqlx::Error> {
    sqlx::query_as!(Location, "SELECT * FROM de WHERE key = $1", id)
        .fetch_optional(pool)
        .await
}

async fn get_from_db(
    pool: &PgPool,
    id: &str,
    start_after: &DateTime<Utc>,
    end_before: &DateTime<Utc>,
) -> Result<Vec<Event>, crate::BoxedError> {
    let events = sqlx::query_as!(Event, r#"SELECT id,room_code,start_at,end_at,stp_title_de,stp_title_en,stp_type,entry_type AS "entry_type!:crate::calendar::models::EventType",detailed_entry_type
            FROM calendar
            WHERE room_code = $1 AND start_at >= $2 AND end_at <= $3"#,
            id, start_after, end_before)
        .fetch_all(pool)
        .await?;
    Ok(events)
}
