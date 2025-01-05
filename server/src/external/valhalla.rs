use tracing::debug;
use valhalla_client::costing::Costing;
use valhalla_client::route::Location;
use valhalla_client::{route, Valhalla};

pub struct ValhallaWrapper {}

impl ValhallaWrapper {
    pub fn route(
        from: valhalla_client::Coordinate,
        to: valhalla_client::Coordinate,
        costing: Costing,
        should_use_english: bool,
    ) -> anyhow::Result<route::Trip> {
        debug!(?from, ?to, "routing request");
        let base_url = "https://nav.tum.de/valhalla".parse()?;
        let valhalla = Valhalla::new(base_url);
        let request = route::Manifest::builder()
            .locations([Location::from(from), Location::from(to)])
            .costing(costing)
            .language(if should_use_english { "en-US" } else { "de-DE" });
        Ok(valhalla.route(request)?)
    }
}
