use serde::{Deserialize, Serialize};

/// departure or arrival event at a stop
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopTime {
    pub agency_id: String,
    pub agency_name: String,
    pub agency_url: String,
    /// For transit legs, the headsign of the bus or train being used.
    /// For non-transit legs, null
    pub headsign: String,
    /// Transport mode for this leg
    pub mode: crate::models::Mode,
    /// information about the stop place and time
    pub place: crate::models::Place,
    /// Whether there is real-time data about this leg
    pub real_time: bool,
    pub route_color: Option<String>,
    pub route_short_name: String,
    pub route_text_color: Option<String>,
    /// Filename and line number where this trip is from
    pub source: String,
    pub trip_id: String,
}
