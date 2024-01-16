mod models;

use std::error::Error;
use std::ops::Sub;
use actix_web::{get, HttpResponse, web};
use serde::Deserialize;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use log::error;
use sqlx::PgPool;
use crate::models::Location;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl};
use oauth2::reqwest::async_http_client;
use std::env;

#[derive(Deserialize, Debug)]
pub struct QueryArguments {
    /// eg. 2039-01-19T03:14:07
    start: NaiveDateTime,
    /// eg. 2042-01-07T00:00:00
    end: NaiveDateTime,
}

const TWO_HOURS: FixedOffset = FixedOffset::east_opt(60 * 60 * 2).unwrap();

fn has_to_refetch(last_requests: &DateTime<Utc>) -> bool {
    let refetch_if_not_done_since = Utc::now().sub(TWO_HOURS);
    last_requests < &refetch_if_not_done_since
}


async fn refetch_calendar_for(id: &str, pool: &PgPool) -> Result<DateTime<Utc>, Box<dyn Error + Send + Sync>> {
    // fetch entries

    match async_http_client()
    let client = BasicClient::new(
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
        if let Err(e) = sqlx::query!(
            r#"INSERT INTO calendar (...)
            VALUES (...)
            ON CONFLICT (key) DO UPDATE SET
             ..."#,
            self.alias,
            self.key,
            self.r#type,
            self.visible_id,
        ).execute(&mut tx).await {
            error!("could not insert {event:?} ({i}/{total}) ignoring",total=events.len());
        }
    }
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
    let last_sync = if !has_to_refetch(last_sync) {
        match refetch_calendar_for(&id, &data.db).await {
            Ok(refetch_time) => {
                data.last_calendar_requests.write().await.insert(id, refetch_time);
                refetch_time
            }
            Err(e) => {
                error!("could not refetch due to {e:?}");
                return HttpResponse::InternalServerError().body("could not get calendar entrys, please try again later");
            }
        }
    } else { last_sync.clone() };


    HttpResponse::Ok().json(models::Events { events, last_sync, calendar_url })
}