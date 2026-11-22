use std::env;
use std::fmt::Debug;

use tracing::debug;
use url::Url;
use valhalla_client::costing::Costing;
use valhalla_client::route::Location;
use valhalla_client::{Units, Valhalla, route};

#[derive(Clone, Debug)]
pub struct ValhallaWrapper(Valhalla);

impl Default for ValhallaWrapper {
    fn default() -> Self {
        let base_url = env::var("VALHALLA_URL")
            .unwrap_or_else(|_| "https://nav.tum.de/valhalla".to_string())
            .parse()
            .expect("Valhalla url is not a valid url");
        Self::new(base_url)
    }
}

impl ValhallaWrapper {
    pub(crate) fn new(base_url: Url) -> Self {
        Self(Valhalla::new(base_url))
    }

    pub async fn route(
        &self,
        from: valhalla_client::Coordinate,
        to: valhalla_client::Coordinate,
        costing: Costing,
        should_use_english: bool,
    ) -> Result<route::Trip, valhalla_client::Error> {
        debug!(?from, ?to, "routing request");
        let request = route::Manifest::builder()
            .locations([
                Location::from(from).radius(20),
                Location::from(to).radius(20),
            ])
            .costing(costing)
            .units(Units::Metric)
            .language(if should_use_english { "en-US" } else { "de-DE" });
        self.0.route(request).await
    }
}
