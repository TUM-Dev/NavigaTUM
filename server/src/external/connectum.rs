const KEEP_ALIVE: Duration = Duration::from_secs(30);
use chrono::{DateTime, Utc};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::url::Url;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::error;

#[derive(Clone)]
pub struct APIRequestor {
    client: reqwest::Client,
    oauth_token: OauthAccessToken,
}
impl Debug for APIRequestor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base = f.debug_struct("APIRequestor");
        if !self.oauth_token.should_refresh_token() {
            base.field("token", &self.oauth_token);
        }
        base.finish()
    }
}
impl Default for APIRequestor {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .tcp_keepalive(KEEP_ALIVE)
            .http2_keep_alive_while_idle(true)
            .http2_keep_alive_interval(KEEP_ALIVE)
            .gzip(true)
            .build()
            .expect("the request client builder is correctly configured");
        Self {
            client,
            oauth_token: OauthAccessToken::new(),
        }
    }
}
impl APIRequestor {
    pub async fn list_events(&mut self, id: &str) -> anyhow::Result<Vec<ConnectumEvent>> {
        let token = self.oauth_token.get_possibly_refreshed_token().await;

        let url = format!("https://campus.tum.de/tumonline/co/connectum/api/rooms/{id}/calendars");

        let events = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?
            .json::<Vec<ConnectumEvent>>()
            .await?;
        Ok(events)
    }
}

#[derive(Deserialize)]
pub struct ConnectumEvent {
    pub id: i32,
    pub room_code: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub title_de: String,
    pub title_en: String,
    pub stp_type: Option<String>,
    pub entry_type: String,
    pub detailed_entry_type: String,
}
#[derive(Clone)]
struct OauthAccessToken(Arc<RwLock<Option<(Instant, BasicTokenResponse)>>>);

impl OauthAccessToken {
    fn new() -> Self {
        Self(Arc::new(RwLock::new(None)))
    }
    fn should_refresh_token(&self) -> bool {
        let token = &self.0.read().expect("lock is not poisoned");
        token.as_ref().is_none_or(|(start, token)| {
            //expires_in ^= how long until it expires. Pretty misleading
            token.expires_in().is_none_or(|expires_in| {
                (expires_in < start.elapsed())
                    || (expires_in - start.elapsed() < Duration::from_secs(30))
            })
        })
    }
    #[tracing::instrument(ret(level = tracing::Level::TRACE))]
    async fn try_refresh_token(&self) -> anyhow::Result<String> {
        if self.should_refresh_token() {
            let new_token = Self::fetch_new_oauth_token().await?;
            let mut token = self.0.write().expect("lock is not poisoned");
            token.replace(new_token);
        }
        Ok(self.unwrap_token())
    }
    fn unwrap_token(&self) -> String {
        let token = self.0.read().expect("lock is not poisoned");
        token
            .as_ref()
            .expect("the token has been set in the last step")
            .1
            .access_token()
            .secret()
            .clone()
    }

    #[tracing::instrument(ret(level = tracing::Level::TRACE))]
    async fn fetch_new_oauth_token() -> anyhow::Result<(Instant, BasicTokenResponse)> {
        let client_id = std::env::var("CONNECTUM_OAUTH_CLIENT_ID")
          .map_err(|e| anyhow::anyhow!("cannot get environment variable CONNECTUM_OAUTH_CLIENT_ID to use this endpoint: {e:?}"))?
          .trim()
          .to_string();
        if client_id.is_empty() {
            anyhow::bail!("environment variable CONNECTUM_OAUTH_CLIENT_ID is present, but empty. It is necessary to use this endpoint")
        }
        let client_secret = std::env::var("CONNECTUM_OAUTH_CLIENT_SECRET")
          .map_err(|e| anyhow::anyhow!("cannot get environment variable CONNECTUM_OAUTH_CLIENT_SECRET to use this endpoint: {e:?}"))?
          .trim()
          .to_string();
        if client_secret.is_empty() {
            anyhow::bail!("environment variable CONNECTUM_OAUTH_CLIENT_ID is present, but empty. It is necessary to use this endpoint")
        }

        // for urls see https://campus.tum.de/tumonline/co/public/sec/auth/isalms/CAMPUSonline_SP/.well-known/openid-configuration
        let auth_url = Url::parse("https://campus.tum.de/tumonline/co/public/sec/auth/realms/CAMPUSonline_SP/protocol/openid-connect/auth")?;
        let token_url = Url::parse("https://campus.tum.de/tumonline/co/public/sec/auth/realms/CAMPUSonline_SP/protocol/openid-connect/token")?;

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

    async fn get_possibly_refreshed_token(&self) -> String {
        let mut token = self.try_refresh_token().await;
        while let Err(e) = token {
            error!("retrying to get oauth token because {e:?}");
            sleep(Duration::from_secs(10)).await;
            token = self.try_refresh_token().await;
        }
        token.unwrap()
    }
}
impl Debug for OauthAccessToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let token = self.0.read().expect("not poisoned");
        let start_elapsed = token.as_ref().map(|(start, _)| start.elapsed());
        let mut base = f.debug_struct("Token");
        if let Some(start_elapsed) = start_elapsed {
            base.field("started_ago", &start_elapsed);
        }
        base.finish()
    }
}
