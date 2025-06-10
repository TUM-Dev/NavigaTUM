use motis_openapi_progenitor::Client;
use std::fmt::Debug;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct MotisWrapper(Client);

impl Default for MotisWrapper {
    fn default() -> Self {
        let base_url =
            std::env::var("MOTIS_URL").unwrap_or_else(|_| "https://api.transitous.org".to_string());
        MotisWrapper(Client::new(&base_url))
    }
}

impl MotisWrapper {
    pub async fn plan(
        &self,
        from: &str,
        to: &str,
    ) -> anyhow::Result<motis_openapi_progenitor::types::PlanResponse> {
        debug!(?from, ?to, "routing request");
        let response = self
            .0
            .plan()
            .from_place(from)
            .to_place(to)
            .detailed_transfers(false)
            .passengers(1)
            .max_pre_transit_time(60 * 30)
            .max_post_transit_time(60 * 30)
            .num_itineraries(5)
            .send()
            .await?;
        Ok(response.into_inner())
    }
    pub async fn stoptimes(
        &self,
        stop_id: &str,
    ) -> anyhow::Result<motis_openapi_progenitor::types::StoptimesResponse> {
        debug!(?stop_id, "stoptimes request");
        let response = self.0.stoptimes().stop_id(stop_id).send().await?;
        Ok(response.into_inner())
    }
}
