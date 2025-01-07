use std::fmt::{Debug, Formatter};
use tracing::debug;
use valhalla_client::costing::Costing;
use valhalla_client::route::Location;
use valhalla_client::{route, Valhalla};

#[derive(Clone, Debug)]
pub struct ValhallaWrapper(Valhalla);

impl ValhallaWrapper {
    pub fn new() -> Self {
        let base_url = "https://nav.tum.de/valhalla".parse().unwrap();
        ValhallaWrapper(Valhalla::new(base_url))
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
            .locations([Location::from(from), Location::from(to)])
            .costing(costing)
            .language(if should_use_english { "en-US" } else { "de-DE" });
        Ok(self.0.route(request).await?)
    }
}
