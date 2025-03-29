use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Place {
    /// arrival time
    pub arrival: Option<String>,
    /// departure time
    pub departure: Option<String>,
    /// latitude
    pub lat: f64,
    /// level according to OpenStreetMap
    pub level: f64,
    /// longitude
    pub lon: f64,
    /// name of the transit stop / PoI / address
    pub name: String,
    /// scheduled arrival time
    #[serde(rename = "scheduledArrival")]
    pub scheduled_arrival: Option<String>,
    /// scheduled departure time
    #[serde(rename = "scheduledDeparture")]
    pub scheduled_departure: Option<String>,
    /// scheduled track from the static schedule timetable dataset
    #[serde(rename = "scheduledTrack")]
    pub scheduled_track: Option<String>,
    /// The ID of the stop. This is often something that users don't care about.
    #[serde(rename = "stopId")]
    pub stop_id: Option<String>,
    /// The current track/platform information, updated with real-time updates if available.
    /// Can be missing if neither real-time updates nor the schedule timetable contains track information.
    pub track: Option<String>,
    #[serde(rename = "vertexType")]
    pub vertex_type: Option<crate::models::VertexType>,
}
