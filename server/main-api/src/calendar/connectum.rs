use std::time::{Duration, Instant};
use std::{env, fmt, io};

use chrono::{DateTime, Utc};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::url::Url;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use sqlx::PgPool;
use tracing::{debug, error, warn};

use crate::calendar::models::Event;
use crate::limited::vec::LimitedVec;

pub(in crate::calendar) struct APIRequestor {
    client: reqwest::Client,
    pool: PgPool,
    oauth_token: Option<(Instant, BasicTokenResponse)>,
}
impl fmt::Debug for APIRequestor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("APIRequestor")
            .field("oauth_token", &self.oauth_token.clone().map(|(i, _)| i))
            .finish()
    }
}

impl From<&PgPool> for APIRequestor {
    fn from(pool: &PgPool) -> Self {
        let keep_alive = Duration::from_secs(30);
        let client = reqwest::Client::builder()
            .tcp_keepalive(keep_alive)
            .http2_keep_alive_while_idle(true)
            .http2_keep_alive_interval(keep_alive)
            .gzip(true)
            .zstd(true)
            .brotli(true)
            .deflate(true)
            .build()
            .expect("the request client builder is correctly configured");
        Self {
            client,
            pool: pool.clone(),
            oauth_token: None,
        }
    }
}

impl APIRequestor {
    #[tracing::instrument]
    pub(crate) async fn refresh(&self, id: String) -> Result<(), crate::BoxedError> {
        let sync_start = Utc::now();
        let url = format!("https://campus.tum.de/tumonline/co/connectum/api/rooms/{id}/calendars");
        let events: Vec<Event> = self
            .client
            .get(url)
            .bearer_auth(self.unwrap_token())
            .send()
            .await?
            .json()
            .await?;
        debug!(
            "finished fetching for {cnt} calendar events of {id}",
            cnt = events.len(),
        );
        let events = events
            .into_iter()
            .map(|mut e| {
                e.room_code.clone_from(&id);
                e
            })
            .collect::<LimitedVec<Event>>();
        self.store(events, &sync_start, &id).await?;
        Ok(())
    }
    fn should_refresh_token(&self) -> bool {
        if let Some((start, token)) = &self.oauth_token {
            if let Some(expires_in) = token.expires_in() {
                return (expires_in < start.elapsed())
                    || (expires_in - start.elapsed() < Duration::from_secs(60));
            }
        }
        true
    }
    #[tracing::instrument(ret(level = tracing::Level::TRACE))]
    pub(crate) async fn try_refresh_token(&mut self) -> Result<String, crate::BoxedError> {
        if self.should_refresh_token() {
            self.oauth_token = Some(Self::fetch_new_oauth_token().await?);
        }

        let at = self
            .oauth_token
            .as_ref()
            .expect("the token has been set in the last step")
            .1
            .access_token();
        Ok(at.secret().clone())
    }
    fn unwrap_token(&self) -> String {
        self.oauth_token
            .as_ref()
            .expect("the token has been set in the last step")
            .1
            .access_token()
            .secret()
            .clone()
    }
}

impl APIRequestor {
    #[tracing::instrument]
    async fn store(
        &self,
        events: LimitedVec<Event>,
        last_calendar_scrape_at: &DateTime<Utc>,
        id: &str,
    ) -> Result<(), crate::BoxedError> {
        // insert into db
        let mut tx = self.pool.begin().await?;
        if let Err(e) = self.delete_events(&mut tx, id).await {
            error!("could not delete existing events because {e:?}");
            tx.rollback().await?;
            return Err(e.into());
        }
        let mut failed: Option<(usize, sqlx::Error)> = None;
        for event in events.0.iter() {
            // conflicts cannot occur because all values for said room were dropped
            if let Err(e) = event.store(&mut tx).await {
                failed = match failed {
                    Some((i, e0)) => Some((i + 1, e0)),
                    None => Some((1, e)),
                };
            }
        }
        if let Some((cnt, e)) = failed {
            warn!(
                "{cnt}/{total} events could not be inserted because of {e:?}",
                total = events.len()
            );
        }
        if let Err(e) = self
            .update_last_calendar_scrape_at(&mut tx, id, last_calendar_scrape_at)
            .await
        {
            error!("could not update last_calendar_scrape_at because {e:?}");
            tx.rollback().await?;
            return Err(e.into());
        }
        tx.commit().await?;
        debug!("finished inserting into the db for {id}");
        Ok(())
    }

    #[tracing::instrument(ret(level = tracing::Level::TRACE))]
    async fn fetch_new_oauth_token() -> Result<(Instant, BasicTokenResponse), crate::BoxedError> {
        let client_id = env::var("CONNECTUM_OAUTH_CLIENT_ID")
            .map_err(|e| {
                error!("CONNECTUM_OAUTH_CLIENT_ID needs to be set: {e:?}");
                io::Error::other("please configure the environment variable CONNECTUM_OAUTH_CLIENT_ID to use this endpoint")
            })?
            .trim().into();
        let client_secret = env::var("CONNECTUM_OAUTH_CLIENT_SECRET")
            .map_err(|e| {
                error!("CONNECTUM_OAUTH_CLIENT_SECRET needs to be set: {e:?}");
                io::Error::other("please configure the environment variable CONNECTUM_OAUTH_CLIENT_SECRET to use this endpoint")
            })?
            .trim().into();

        // for urls see https://campus.tum.de/tumonline/co/public/sec/auth/realms/CAMPUSonline/.well-known/openid-configuration
        let auth_url = Url::parse("https://campus.tum.de/tumonline/co/public/sec/auth/realms/CAMPUSonline/protocol/openid-connect/auth")?;
        let token_url = Url::parse("https://campus.tum.de/tumonline/co/public/sec/auth/realms/CAMPUSonline/protocol/openid-connect/token")?;

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
        Ok((Instant::now(), token?))
    }
    #[tracing::instrument(skip(tx))]
    async fn delete_events(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &str,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"DELETE FROM calendar WHERE room_code = $1"#, id)
            .execute(&mut **tx)
            .await
    }
    #[tracing::instrument(skip(tx))]
    async fn update_last_calendar_scrape_at(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &str,
        last_calendar_scrape_at: &DateTime<Utc>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "UPDATE en SET last_calendar_scrape_at = $1 WHERE key=$2",
            last_calendar_scrape_at,
            id
        )
        .execute(&mut **tx)
        .await?;
        sqlx::query!(
            "UPDATE de SET last_calendar_scrape_at = $1 WHERE key=$2",
            last_calendar_scrape_at,
            id
        )
        .execute(&mut **tx)
        .await
    }
}
