use motis_openapi_progenitor::{Client, types::PlanResponse};
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
    pub async fn route(
        &self,
        from: &str,
        to: &str,
        page_cursor: Option<&str>,
        time: Option<&chrono::DateTime<chrono::Utc>>,
        arrive_by: bool,
    ) -> anyhow::Result<PlanResponse> {
        debug!(?from, ?to, "routing request");
        let mut request = self
            .0
            .plan()
            .detailed_transfers(true)
            .num_itineraries(5)
            .from_place(from)
            .to_place(to);
        if let Some(cursor) = page_cursor {
            request = request.page_cursor(cursor);
        }
        if let Some(time) = time {
            request = request.time(*time).arrive_by(arrive_by);
        }

        Ok(request.send().await?.into_inner())
    }
}
