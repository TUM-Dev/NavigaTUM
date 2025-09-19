use motis_openapi_progenitor::{Client, types::PlanResponse};
use std::fmt::Debug;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct MotisWrapper(Client);

impl Default for MotisWrapper {
    fn default() -> Self {
        let base_url = std::env::var("MOTIS_URL")
            .unwrap_or_else(|_| "https://api.transitous.org/api/".to_string());
        MotisWrapper(Client::new(&base_url))
    }
}

impl MotisWrapper {
    pub async fn route(&self, from: &str, to: &str) -> anyhow::Result<PlanResponse> {
        debug!(?from, ?to, "routing request");
        let result = self
            .0
            .plan()
            .from_place(from)
            .to_place(to)
            .send()
            .await?
            .into_inner();
        Ok(result)
    }
}
