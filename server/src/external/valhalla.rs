use std::env;
use std::fmt::Debug;

use tracing::debug;
use url::Url;
use valhalla_client::costing::Costing;
use valhalla_client::route::Location;
use valhalla_client::{Units, Valhalla, route};

mod oberbayern;

const DEFAULT_URL: &str = "https://nav.tum.de/valhalla";

#[derive(Clone, Debug)]
pub struct ValhallaWrapper {
    client: Valhalla,
    uses_oberbayern_extract: bool,
}

impl Default for ValhallaWrapper {
    fn default() -> Self {
        let url = env::var("VALHALLA_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
        Self::new(url.parse().expect("Valhalla url is not a valid url"))
    }
}

impl ValhallaWrapper {
    pub(crate) fn new(base_url: Url) -> Self {
        let uses_oberbayern_extract = base_url.as_str().trim_end_matches('/') == DEFAULT_URL;
        Self {
            client: Valhalla::new(base_url),
            uses_oberbayern_extract,
        }
    }

    pub fn covers(&self, lat: f64, lon: f64) -> bool {
        !self.uses_oberbayern_extract || oberbayern::contains(lat, lon)
    }

    pub async fn route(
        &self,
        from: valhalla_client::Coordinate,
        to: valhalla_client::Coordinate,
        costing: Costing,
        should_use_english: bool,
    ) -> anyhow::Result<route::Trip> {
        debug!(?from, ?to, "routing request");
        let request = route::Manifest::builder()
            .locations([
                Location::from(from).radius(20),
                Location::from(to).radius(20),
            ])
            .costing(costing)
            .units(Units::Metric)
            .language(if should_use_english { "en-US" } else { "de-DE" });
        Ok(self.client.route(request).await?)
    }
}
