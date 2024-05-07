use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use log::error;
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::Location;

mod fetch;
mod models;

async fn get_location(pool: &PgPool, id: &str) -> Result<Option<Location>, sqlx::Error> {
    sqlx::query_as!(Location, "SELECT * FROM de WHERE key = $1", id)
        .fetch_optional(pool)
        .await
}

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
                .body("could not get calendar entrys, please try again later");
        }
        Ok(None) => {
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Room not found");
        }
        Ok(Some(loc)) => loc,
    };
    let Some(calendar_url) = location.calendar_url else {
        return HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Room does not have a calendar");
    };
    let fetching_strategy =
        fetch::StrategyExecutor::new(&data.db, &id, &args.start_after, &args.end_before);
    match fetching_strategy
        .exec_with_retrying(&location.last_calendar_scrape_at)
        .await
    {
        Ok((last_sync, events)) => HttpResponse::Ok().json(models::Events {
            events,
            last_sync,
            calendar_url,
        }),
        Err(resp) => resp,
    }
}
