use std::fmt::Debug;
use tracing::debug;
use valhalla_client::costing::Costing;
use valhalla_client::route::Location;
use valhalla_client::{Units, Valhalla, route};

#[derive(Clone, Debug)]
pub struct ValhallaWrapper(Valhalla);

impl Default for ValhallaWrapper {
    fn default() -> Self {
        let base_url = std::env::var("VALHALLA_URL")
            .unwrap_or_else(|_| "https://nav.tum.de/valhalla".to_string())
            .parse()
            .expect("Valhalla url is not a valid url");
        ValhallaWrapper(Valhalla::new(base_url))
    }
}

impl ValhallaWrapper {
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
        Ok(self.0.route(request).await?)
    }
}
