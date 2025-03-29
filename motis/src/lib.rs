#![forbid(unsafe_code)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

/// models used in the various apis
pub mod models;
mod routing;
pub use routing::*;

use tracing::trace;
const MOTIS_PUBLIC_API_URL: &str = "https://api.transitous.org";

/// synchronous ("blocking") client implementation
#[cfg(feature = "blocking")]
pub mod blocking {
    use super::MOTIS_PUBLIC_API_URL;
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    pub struct Motis {
        runtime: Arc<tokio::runtime::Runtime>,
        client: super::Motis,
    }
    impl Motis {
        /// Create a sync [Motis](https://motis-project.de/) client
        pub fn new(base_url: url::Url) -> Self {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build()
                .expect("tokio runtime can be created");
            Self {
                runtime: Arc::new(runtime),
                client: super::Motis::new(base_url),
            }
        }

        // recreate api here
        //pub fn TODO(&self, manifest: matrix::Manifest) -> Result<matrix::Response, Error> {
        //    self.runtime
        //        .block_on(async move { self.client.matrix(manifest).await })
        //}
    }
    impl Default for Motis {
        fn default() -> Self {
            Self::new(
                url::Url::parse(MOTIS_PUBLIC_API_URL)
                    .expect("MOTIS_PUBLIC_API_URL is not a valid url"),
            )
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct MotisError {
    pub error: String,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Url(url::ParseError),
    Serde(serde_json::Error),
    RemoteError(MotisError),
}

#[derive(Debug, Clone)]
pub struct Motis {
    client: reqwest::Client,
    base_url: url::Url,
}
impl Motis {
    /// Create an async [Motis](https://motis-project.de/) client
    pub fn new(base_url: url::Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }
    
    pub fn optimal_connections(&self,options:routing::OptimalConnectionOptions)->Result<routing::OptimalConnectionResponse,Error>{
      self.do_request(options, "/api/v1/plan", "optimal_connections")
    }
    async fn do_request<Resp: for<'de> serde::Deserialize<'de>>(
        &self,
        body: impl serde::Serialize,
        path: &'static str,
        name: &'static str,
    ) -> Result<Resp, Error> {
        if tracing::event_enabled!(tracing::Level::TRACE) {
            let request = serde_json::to_string(&body).unwrap();
            trace!("Sending {name} request: {request}");
        }
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base_url is not a valid base url")
            .push(path);
        let response = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(Error::Reqwest)?;
        if response.status().is_client_error() {
            return Err(Error::RemoteError(
                response.json().await.map_err(Error::Reqwest)?,
            ));
        }
        response.error_for_status_ref().map_err(Error::Reqwest)?;
        let text = response.text().await.map_err(Error::Reqwest)?;
        trace!("{name} responded: {text}");
        let response: Resp = serde_json::from_str(&text).map_err(Error::Serde)?;
        Ok(response)
    }
}

impl Default for Motis {
    fn default() -> Self {
        Self::new(
            url::Url::parse(MOTIS_PUBLIC_API_URL).expect("MOTIS_PUBLIC_API_URL is not a valid url"),
        )
    }
}
