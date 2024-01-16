mod models;

use std::any::Any;
use std::error::Error;
use std::ops::Sub;
use actix_web::{get, HttpResponse, web};
use serde::Deserialize;
use chrono::{DateTime, FixedOffset, Local};
use log::error;
use sqlx::PgPool;
use crate::models::Location;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenUrl};
use std::env;

#[derive(Deserialize, Debug)]
pub struct QueryArguments {
    /// eg. 2039-01-19T03:14:07
    start_after: DateTime<Local>,
    /// eg. 2042-01-07T00:00:00
    end_before: DateTime<Local>,
}

const ONE_HOUR: FixedOffset = FixedOffset::east_opt(60 * 60 * 1).unwrap();
const THREE_DAYS: FixedOffset = FixedOffset::east_opt(60 * 60 * 3).unwrap();

fn has_to_refetch(last_requests: &DateTime<Local>) -> bool {
    let refetch_if_not_done_after = Local::now().sub(ONE_HOUR);
    last_requests < &refetch_if_not_done_after
}

fn can_use_stale_result_from_db(last_requests: &DateTime<Local>) -> bool {
    let can_reuse_if_done_after = Local::now().sub(THREE_DAYS);
    last_requests < &can_reuse_if_done_after
}

async fn refetch_calendar_for(id: &str, pool: &PgPool) -> Result<(DateTime<Local>, Vec<models::Event>), Box<dyn Error + Send + Sync>> {
    // setup clients
    let oauth2_client = BasicClient::new(
        ClientId::new(env::var("TUMONLINE_OAUTH_CLIENT_ID")?),
        Some(ClientSecret::new(env::var("TUMONLINE_OAUTH_CLIENT_SECRET")?)),
        AuthUrl::new("https://review.campus.tum.de/RSYSTEM/co/public/sec/auth/realms/CAMPUSonline".to_string())?,
        Some(TokenUrl::new("https://example.com/token".to_string())?),
    );

    // Make OAuth2 secured request
    let auth_url = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("connectum-rooms.read".into()))
        .url();
    let events: Vec<models::Event> = reqwest::get(format!("https://review.campus.tum.de/RSYSTEM/co/connectum/api/rooms/{id}/calendar")).await?.json().await?;
    // insert into db
    let mut tx = pool.begin().await?;
    if let Err(e) = sqlx::query!(r#"DROP FROM calendar WHERE key = $1"#, id).execute(&mut tx).await {
        tx.rollback().await?;
    }
    for (i, event) in events.iter().enumerate() {
        // conflicts cannot occur because all values for said room were dropped
        if let Err(e) = sqlx::query!(
            r#"INSERT INTO calendar (id,room_code,start,end,stp_title_de,stp_title_en,stp_type,entry_type,detailed_entry_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            event.id,
            event.room_code,
            event.start,
            event.end,
            event.stp_title_de,
            event.stp_title_en,
            event.stp_type,
            event.entry_type,
            event.detailed_entry_type,
        ).execute(&mut tx).await {
            error!("could not insert {event:?} ({i}/{total}) ignoring",total=events.len());
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

async fn get_events_from_db(pool: &PgPool, id: &str, start_after: &DateTime<Local>, end_before: &DateTime<Local>) -> Result<Vec<models::Event>, sqlx::Error> {
    sqlx::query_as!(models::Event, "SELECT * FROM calendar WHERE room_code = $1 AND start_at >= $2 AND end_at <= $3", id, start_after, end_before)
        .fetch_all(pool)
        .await
}

#[get("/api/calendar/{id}")]
pub async fn calendar_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<QueryArguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params.into_inner();
    let room = match get_location(&data.db, &id).await {
        Err(e) => {
            error!("could not refetch due to {e:?}");
            return HttpResponse::InternalServerError().body("could not get calendar entrys, please try again later");
        }
        Ok(None) => {
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Room not found");
        }
        Ok(Some(loc)) => loc,
    };
    let calendar_url = format!("https://campus.tum.de/tumonline/wbKalender.wbRessource?pResNr={id}", id = room.tumonline_calendar_id);

    let last_sync = data.last_calendar_requests.read().await.get(&id).unwrap_or(&DateTime::default());
    let (last_sync, events) = if !has_to_refetch(last_sync) {
        match refetch_calendar_for(&id, &data.db).await {
            Ok((last_sync, events)) => {
                data.last_calendar_requests.write().await.insert(id, last_sync);
                let events = events.into_iter().filter(|e| args.start_after <= e.start_at && args.end_before >= e.end_at).collect::<Vec<models::Event>>();
                (last_sync, events)
            }
            Err(e) => {
                error!("could not refetch due to {e:?}");
                if can_use_stale_result_from_db(last_sync) {
                    match get_events_from_db(&data.db, &id, &args.start_after, &args.end_before).await {
                        Ok(res) => (last_sync.clone(), res),
                        Err(e) => {
                            error!("could substitute from db {e:?}");
                            return HttpResponse::InternalServerError().body("could not get calendar entrys, please try again later");
                        }
                    }
                } else {
                    error!("could substitute from db {e:?}");
                    return HttpResponse::InternalServerError().body("could not get calendar entrys, please try again later");
                }
            }
        }
    } else {
        match get_events_from_db(&data.db, &id, &args.start_after, &args.end_before).await {
            Ok(res) => (last_sync.clone(), res),
            Err(e) => {
                error!("could not refetch due to {e:?}");
                return HttpResponse::InternalServerError().body("could not get calendar entrys, please try again later");
            }
        }
    };

    HttpResponse::Ok().json(models::Events { events, last_sync, calendar_url })
}