use crate::limited::vec::LimitedVec;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct NominatimAddressResponse {
    // postcode: Option<String>,
    // country: Option<String>,
    // country_code: Option<String>,
    // ISO3166-2-lvl4: Option<String>,
    state: Option<String>,
    county: Option<String>,
    town: Option<String>,
    suburb: Option<String>,
    village: Option<String>,
    hamlet: Option<String>,
    pub road: Option<String>,
}

impl NominatimAddressResponse {
    pub fn serialise(&self) -> String {
        let mut result = Vec::<String>::new();
        if let Some(state) = self.state.clone() {
            result.push(state);
        }
        if let Some(county) = self.county.clone() {
            result.push(county);
        }
        if let Some(town) = self.town.clone() {
            result.push(town);
        }
        if let Some(suburb) = self.suburb.clone() {
            result.push(suburb);
        }
        if let Some(village) = self.village.clone() {
            result.push(village);
        }
        if let Some(hamlet) = self.hamlet.clone() {
            result.push(hamlet);
        }
        if let Some(road) = self.road.clone() {
            result.push(road);
        }
        result.join(", ")
    }
}

#[derive(Deserialize, Clone)]
pub struct Nominatim {
    /// Example: 371651568
    pub osm_id: i64,
    /// Example: "road",
    #[serde(rename = "addresstype")]
    pub address_type: String,
    /// Example: "Münchner Straße",
    pub name: String,
    pub address: NominatimAddressResponse,
}
impl Nominatim {
    #[tracing::instrument]
    pub async fn address_search(q: &str) -> anyhow::Result<LimitedVec<Self>> {
        let url = std::env::var("NOMINATIM_URL")
            .unwrap_or_else(|_| "https://nav.tum.de/nominatim".to_string());
        let url = format!("{url}/search?q={q}&addressdetails=1");
        let Ok(nominatim_results) = reqwest::get(&url).await else {
            anyhow::bail!("cannot get {url}");
        };
        let Ok(results) = nominatim_results.json::<Vec<Self>>().await else {
            anyhow::bail!("the results from nomnatim is not what we expected {url}");
        };
        Ok(LimitedVec(results))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn serialize_address() {
        let response = NominatimAddressResponse {
            state: None,
            county: None,
            town: None,
            suburb: None,
            village: None,
            hamlet: None,
            road: None,
        };
        insta::assert_snapshot!(response.serialise(), @"");
        let response = NominatimAddressResponse {
            state: Some("Bavaria".to_string()),
            county: Some("Germany".to_string()),
            town: Some("Berlin".to_string()),
            suburb: Some("Neuköln".to_string()),
            village: None,
            hamlet: None,
            road: Some("Münchnerstraße 21".to_string()),
        };
        insta::assert_snapshot!(response.serialise(), @"Bavaria, Germany, Berlin, Neuköln, Münchnerstraße 21");
    }
}
