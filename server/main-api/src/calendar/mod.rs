mod models;

use crate::models::Location;
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, FixedOffset, Local};
use log::error;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use serde::Deserialize;
use sqlx::PgPool;
use std::env;
use std::error::Error;
use std::ops::Sub;

fn has_to_refetch(last_requests: &DateTime<Local>) -> bool {
    let one_hour = FixedOffset::east_opt(60 * 60).expect("time travel is impossible and chronos is 2038-save");
    let refetch_if_not_done_after = Local::now().sub(one_hour);
    &refetch_if_not_done_after < last_requests
}

fn can_use_stale_result_from_db(last_requests: &DateTime<Local>) -> bool {
    let three_days = chrono::Days::new(3);
    let can_reuse_if_done_after = Local::now().checked_sub_days(three_days).expect("time travel is impossible and chronos is 2038-save");
    &can_reuse_if_done_after < last_requests
}

async fn delete_events(
    id: &str,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(r#"DELETE FROM calendar WHERE room_code = $1"#, id)
        .execute(&mut **tx)
        .await
}

async fn fetch_oauth_token() -> Result<BasicTokenResponse, Box<dyn Error + Send + Sync>> {
    let oauth2_client = BasicClient::new(
        ClientId::new(env::var("TUMONLINE_OAUTH_CLIENT_ID")?),
        Some(ClientSecret::new(env::var(
            "TUMONLINE_OAUTH_CLIENT_SECRET",
        )?)),
        AuthUrl::new(
            "https://review.campus.tum.de/RSYSTEM/co/public/sec/auth/realms/CAMPUSonline"
                .to_string(),
        )?,
        Some(TokenUrl::new("https://example.com/token".to_string())?),
    );

    let token = oauth2_client
        .exchange_client_credentials()
        .add_scope(Scope::new("connectum-rooms.read".into()))
        .request_async(async_http_client)
        .await?; // not directly returned for typing issues
    Ok(token)
}

async fn refetch_calendar_for(
    id: &str,
    pool: &PgPool,
) -> Result<(DateTime<Local>, Vec<models::Event>), Box<dyn Error + Send + Sync>> {
    // Make OAuth2 secured request
    let oauth_token = fetch_oauth_token().await?;
    let events: Vec<models::Event> = reqwest::Client::new()
        .get(format!(
            "https://review.campus.tum.de/RSYSTEM/co/connectum/api/rooms/{id}/calendar"
        ))
        .bearer_auth(oauth_token.access_token().secret().clone())
        .send()
        .await?
        .json()
        .await?;
    // insert into db
    let mut tx = pool.begin().await?;
    if let Err(e) = delete_events(id, &mut tx).await {
        error!("could not delete existing events because {e:?}");
        tx.rollback().await?;
        return Err(e.into());
    }
    for (i, event) in events.iter().enumerate() {
        // conflicts cannot occur because all values for said room were dropped
        if let Err(e) = event.store(&mut tx).await {
            error!(
                "ignoring insert {event:?} ({i}/{total}) because {e:?}",
                total = events.len()
            );
        }
    }
    tx.commit().await?;
    Ok((Local::now(), events))
}

async fn get_location(pool: &PgPool, id: &str) -> Result<Option<Location>, sqlx::Error> {
    sqlx::query_as!(Location, "SELECT * FROM en WHERE key = $1", id)
        .fetch_optional(pool)
        .await
}

async fn get_events_from_db(
    pool: &PgPool,
    id: &str,
    start_after: &DateTime<Local>,
    end_before: &DateTime<Local>,
) -> Result<Vec<models::Event>, sqlx::Error> {
    sqlx::query_as!(models::Event, r#"SELECT id,room_code,start_at,end_at,stp_title_de,stp_title_en,stp_type,entry_type AS "entry_type!:models::EventType",detailed_entry_type
    FROM calendar
    WHERE room_code = $1 AND start_at >= $2 AND end_at <= $3"#, id, start_after, end_before)
        .fetch_all(pool)
        .await
}

#[derive(Deserialize, Debug)]
pub struct QueryArguments {
    /// eg. 2039-01-19T03:14:07+1
    start_after: DateTime<Local>,
    /// eg. 2042-01-07T00:00:00 UTC
    end_before: DateTime<Local>,
}

#[get("/api/calendar/{id}")]
pub async fn calendar_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<QueryArguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params.into_inner();
    match get_location(&data.db, &id).await {
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
    let calendar_url = format!(
        "https://campus.tum.de/tumonline/wbKalender.wbRessource?pResNr={id}",
        id = 0
    ); // TODO: room.tumonline_calendar_id

    let sync_times = data.last_calendar_requests.read().await;
    let default_sync_time = DateTime::default();
    let last_sync = sync_times.get(&id).unwrap_or(&default_sync_time);
    let (last_sync, events) = if !has_to_refetch(last_sync) {
        match refetch_calendar_for(&id, &data.db).await {
            Ok((last_sync, events)) => {
                data.last_calendar_requests
                    .write()
                    .await
                    .insert(id, last_sync);
                let events = events
                    .into_iter()
                    .filter(|e| args.start_after <= e.start_at && args.end_before >= e.end_at)
                    .collect::<Vec<models::Event>>();
                (last_sync, events)
            }
            Err(e) => {
                error!("could not refetch due to {e:?}");
                if !can_use_stale_result_from_db(last_sync) {
                    match get_events_from_db(&data.db, &id, &args.start_after, &args.end_before)
                        .await
                    {
                        Ok(res) => (*last_sync, res),
                        Err(e) => {
                            error!("could not get substitute from db due to {e:?}");
                            return HttpResponse::InternalServerError()
                                .body("could not get calendar entrys, please try again later");
                        }
                    }
                } else {
                    error!("cannot get substitute from db due to staleness");
                    return HttpResponse::InternalServerError()
                        .body("could not get calendar entrys, please try again later");
                }
            }
        }
    } else {
        match get_events_from_db(&data.db, &id, &args.start_after, &args.end_before).await {
            Ok(res) => (*last_sync, res),
            Err(e) => {
                error!("could not refetch due to {e:?}");
                return HttpResponse::InternalServerError()
                    .body("could not get calendar entrys, please try again later");
            }
        }
    };

    HttpResponse::Ok().json(models::Events {
        events,
        last_sync,
        calendar_url,
    })
}
