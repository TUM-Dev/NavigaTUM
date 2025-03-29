use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Itinerary {
    /// journey duration in seconds
    pub duration: i64,
    /// journey arrival time
    pub end_time: String,
    /// Journey legs
    pub legs: Vec<crate::models::Leg>,
    /// journey departure time
    pub start_time: String,
    /// The number of transfers this trip has.
    pub transfers: i64,
}
