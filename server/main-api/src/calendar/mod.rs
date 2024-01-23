use std::env;
use std::error::Error;
use std::ops::Sub;

use actix_web::{get, HttpResponse, web};
use cached::instant::Instant;
use chrono::{DateTime, FixedOffset, Local, Utc};
use log::{debug, error, info};
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use reqwest::Url;
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::Location;

mod models;
mod fetcher;
mod fetch;

fn has_to_refetch(last_requests: &Option<DateTime<Utc>>) -> bool {
    let one_hour = FixedOffset::east_opt(60 * 60)
        .expect("time travel is impossible and chronos is Y2K38-safe");
    let refetch_if_not_done_after = Local::now().sub(one_hour);
    match last_requests {
        Some(last) => &refetch_if_not_done_after < last,
        None => true,
    }
}

fn can_use_stale_result_from_db(last_requests: &Option<DateTime<Utc>>) -> bool {
    let three_days = chrono::Days::new(3);
    let can_reuse_if_done_after = Local::now()
        .checked_sub_days(three_days)
        .expect("time travel is impossible and chronos is Y2K38-save");
    match last_requests {
        Some(last) => &can_reuse_if_done_after < last,
        None => false,
    }
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
    let client_id = env::var("TUMONLINE_OAUTH_CLIENT_ID").expect(
        "please configure the environment variable TUMONLINE_OAUTH_CLIENT_ID to use this endpoint",
    );
    let client_secret = env::var("TUMONLINE_OAUTH_CLIENT_SECRET").expect("please configure the environment variable TUMONLINE_OAUTH_CLIENT_SECRET to use this endpoint");

    // for urls see https://review.campus.tum.de/RSYSTEM/co/public/sec/auth/realms/CAMPUSonline/.well-known/openid-configuration
    let auth_url = Url::parse("https://review.campus.tum.de/RSYSTEM/co/public/sec/auth/realms/CAMPUSonline/protocol/openid-connect/auth")?;
    let token_url = Url::parse("https://review.campus.tum.de/RSYSTEM/co/public/sec/auth/realms/CAMPUSonline/protocol/openid-connect/token")?;

    let token = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::from_url(auth_url),
        Some(TokenUrl::from_url(token_url)),
    )
    .exchange_client_credentials()
    .add_scope(Scope::new("connectum-rooms.read".into()))
    .request_async(async_http_client)
    .await;
    Ok(token?) // not directly returned for typing issues
}

async fn refetch_calendar_for(
    tumonline_id: &str,
    id: &str,
    pool: &PgPool,
) -> Result<(DateTime<Utc>, Vec<models::Event>), Box<dyn Error + Send + Sync>> {
    let start = Instant::now();
    // Make OAuth2 secured request
    let oauth_token = fetch_oauth_token().await?;
    let bearer_token = oauth_token.access_token().secret().clone();
    let url = format!("https://review.campus.tum.de/RSYSTEM/co/connectum/api/rooms/{tumonline_id}/calendars");

    let events: Vec<models::Event> = reqwest::Client::new()
        .get(url)
        .bearer_auth(bearer_token)
        .send()
        .await?
        .json()
        .await?;
    info!("finished fetching for {id}: {cnt} calendar events in {elapsed:?}", cnt=events.len(), elapsed=start.elapsed());
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
    debug!("finished inserting into the db for {id}");
    Ok((Utc::now(), events))
}

async fn get_location(pool: &PgPool, id: &str) -> Result<Option<Location>, sqlx::Error> {
    sqlx::query_as!(Location, "SELECT * FROM de WHERE key = $1", id)
        .fetch_optional(pool)
        .await
}

async fn get_events_from_db(
    pool: &PgPool,
    id: &str,
    start_after: &DateTime<Utc>,
    end_before: &DateTime<Utc>,
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
    let id = params.into_inner();
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
    let calendar_url = format!(
        "https://campus.tum.de/tumonline/wbKalender.wbRessource?pResNr={id}",
        id = 0
    ); // TODO: room.tumonline_calendar_id
    let (last_sync, events) = if !has_to_refetch(&location.last_calendar_scrape_at) {
        let tumonline_id = id.clone().replace('.', "");
        match refetch_calendar_for(&tumonline_id, &id, &data.db).await {
            Ok((last_sync, events)) => {
                sqlx::query!("UPDATE de SET last_calendar_scrape_at = $1 WHERE key=$2", last_sync, id).execute(&data.db).await;
                let events = events
                    .into_iter()
                    .filter(|e| args.start_after <= e.start_at && args.end_before >= e.end_at)
                    .collect::<Vec<models::Event>>();
                (last_sync, events)
            }
            Err(e) => {
                error!("could not refetch due to {e:?}");
                if !can_use_stale_result_from_db(&location.last_calendar_scrape_at) {
                    match get_events_from_db(&data.db, &id, &args.start_after, &args.end_before)
                        .await
                    {
                        Ok(res) => (location.last_calendar_scrape_at, res),
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
            Ok(res) => (location.last_calendar_scrape_at, res),
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
