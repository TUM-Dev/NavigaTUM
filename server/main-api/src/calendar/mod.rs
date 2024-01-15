mod models;

use std::error::Error;
use std::ops::Sub;
use actix_web::{get, HttpResponse, web};
use serde::Deserialize;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use log::error;
use sqlx::PgPool;
use crate::models::Location;

#[derive(Deserialize, Debug)]
pub struct QueryArguments {
    start: NaiveDateTime,
    // eg. 2022-01-01T00:00:00
    end: NaiveDateTime,   // eg. 2022-01-07T00:00:00
}

const TWO_HOURS: FixedOffset = FixedOffset::east_opt(60 * 60 * 2).unwrap();

fn has_to_refetch(last_requests: &DateTime<Utc>) -> bool {
    let refetch_if_not_done_since = Utc::now().sub(TWO_HOURS);
    last_requests < &refetch_if_not_done_since
}

async fn refetch_calendar_for(id: &str, pool: &PgPool) -> Result<DateTime<Utc>, Box<dyn Error + Send + Sync>> {
    // fetch entries
reqwest::get("htttps://campus.tum.de/")
    // insert into
    let tx = pool.begin().await?;
    sqlx::query_as!(Location, "SELECT * FROM en WHERE key = $1", id)
        .fetch_optional(pool)
        .await
    tx.commit().await?;
    Ok(Utc::now())
}

async fn get_location(pool: &PgPool, id: &str) -> Result<Option<Location>, sqlx::Error> {
    sqlx::query_as!(Location, "SELECT * FROM en WHERE key = $1", id)
        .fetch_optional(pool)
        .await
}

#[get("/api/calendar/{id}")]
pub async fn calendar_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<QueryArguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params.into_inner();
    let room =match get_location(&data.db,&id).await {
        Err(e)=>{
            error!("could not refetch due to {e:?}");
            return HttpResponse::InternalServerError().body("could not get calendar entrys, please try again later");
        },
        Ok(None)=>{
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Room not found");
        },
        Ok(Some(loc))=>loc,
    };
    let calendar_url= format!("https://campus.tum.de/tumonline/wbKalender.wbRessource?pResNr={id}",id = room.tumonline_calendar_id);

    let last_sync = data.last_calendar_requests.read().await.get(&id).unwrap_or(&DateTime::default());
    let last_sync= if !has_to_refetch(last_sync) {
        match refetch_calendar_for(&id, &data.db).await {
            Ok(refetch_time)=> {
                data.last_calendar_requests.write().await.insert(id, refetch_time);
                refetch_time },
            Err(e) => {
                error!("could not refetch due to {e:?}");
                return HttpResponse::InternalServerError().body("could not get calendar entrys, please try again later");
            }
        }
    } else { last_sync.clone() };


    HttpResponse::Ok().json(models::Events { events,last_sync,calendar_url,})
}