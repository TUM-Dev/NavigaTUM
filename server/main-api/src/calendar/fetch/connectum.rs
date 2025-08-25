use std::env;

use cached::instant::Instant;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::url::Url;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use sqlx::PgPool;

use crate::calendar::fetch::CalendarEntryFetcher;
use crate::calendar::models::Event;

pub(super) struct APIRequestor {
    client: reqwest::Client,
    pool: PgPool,
}

impl CalendarEntryFetcher for APIRequestor {
    fn new(pool: &PgPool, _: &Option<DateTime<Utc>>) -> Self {
        Self {
            client: reqwest::Client::new(),
            pool: pool.clone(),
        }
    }
    async fn fetch(
        &self,
        id: &str,
        start_after: &DateTime<Utc>,
        end_before: &DateTime<Utc>,
    ) -> Result<super::CalendarEntries, crate::BoxedError> {
        let tumonline_id = id.replace('.', "");

        let sync_start = Utc::now();
        let start = Instant::now();
        // Make OAuth2 secured request
        let oauth_token = self
            .fetch_oauth_token()
            .await?
            .access_token()
            .secret()
            .clone();
        let url = format!(
            "https://review.campus.tum.de/RSYSTEM/co/connectum/api/rooms/{tumonline_id}/calendars"
        );

        let events: Vec<Event> = self
            .client
            .get(url)
            .bearer_auth(oauth_token)
            .send()
            .await?
            .json()
            .await?;
        info!(
            "finished fetching for {id}: {cnt} calendar events in {elapsed:?}",
            cnt = events.len(),
            elapsed = start.elapsed()
        );
        let events = events
            .into_iter()
            .map(|mut e| {
                e.room_code = id.into();
                e
            })
            .collect::<Vec<Event>>();
        self.store(&events, &sync_start, id).await?;
        let events = events
            .into_iter()
            .filter(|e| *start_after <= e.start_at && *end_before >= e.end_at)
            .collect();
        Ok((sync_start, events))
    }
}

impl APIRequestor {
    async fn store(
        &self,
        events: &[Event],
        last_sync: &DateTime<Utc>,
        id: &str,
    ) -> Result<(), crate::BoxedError> {
        // insert into db
        let mut tx = self.pool.begin().await?;
        if let Err(e) = self.delete_events(id, &mut tx).await {
            error!("could not delete existing events because {e:?}");
            tx.rollback().await?;
            return Err(e.into());
        }
        for (i, event) in events.iter().enumerate() {
            // conflicts cannot occur because all values for said room were dropped
            if let Err(e) = event.store(&mut tx).await {
                warn!(
                    "ignoring insert {event:?} ({i}/{total}) because {e:?}",
                    total = events.len()
                );
            }
        }
        sqlx::query!(
            "UPDATE de SET last_calendar_scrape_at = $1 WHERE key=$2",
            last_sync,
            id
        )
        .execute(&self.pool)
        .await?;
        tx.commit().await?;
        debug!("finished inserting into the db for {id}");
        Ok(())
    }
    async fn fetch_oauth_token(&self) -> Result<BasicTokenResponse, crate::BoxedError> {
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
    async fn delete_events(
        &self,
        id: &str,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"DELETE FROM calendar WHERE room_code = $1"#, id)
            .execute(&mut **tx)
            .await
    }
}
