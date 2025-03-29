use serde::{Deserialize, Serialize};

/// GeoCoding match
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Match {
    /// list of areas
    pub areas: Vec<crate::models::Area>,
    /// house number
    #[serde(rename = "houseNumber")]
    pub house_number: Option<String>,
    /// unique ID of the location
    pub id: String,
    /// latitude
    pub lat: f64,
    /// level according to OpenStreetMap
    /// (at the moment only for public transport)
    pub level: Option<f64>,
    /// longitude
    pub lon: f64,
    /// name of the location (transit stop / PoI / address)
    pub name: String,
    /// score according to the internal scoring system (the scoring algorithm might change in the future)
    pub score: f64,
    /// street name
    pub street: Option<String>,
    /// list of non-overlapping tokens that were matched
    pub tokens: Vec<Vec<f64>>,
    /// location type
    #[serde(rename = "type")]
    pub r#type: crate::models::MatchType,
    /// zip code
    pub zip: Option<String>,
}

/// location type
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchType {
    #[default]
    Address,
    Place,
    Stop,
}
