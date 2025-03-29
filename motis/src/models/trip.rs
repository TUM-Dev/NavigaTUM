use serde::{Deserialize, Serialize};

/// trip id and name
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripInfo {
    /// trip display name
    pub route_short_name: String,
    /// trip ID (dataset trip id prefixed with the dataset tag)
    pub trip_id: String,
}
/// trip segment between two stops to show a trip on a map
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TripSegment {
    /// arrival time
    pub arrival: String,
    /// departure time
    pub departure: String,
    /// distance in meters
    pub distance: f64,
    pub from: crate::models::Place,
    /// Transport mode for this leg
    pub mode: crate::models::Mode,
    /// Google polyline encoded coordinate sequence (with precision 7) where the trip travels on this segment.
    pub polyline: String,
    /// Whether there is real-time data about this leg
    #[serde(rename = "realTime")]
    pub real_time: bool,
    #[serde(rename = "routeColor")]
    pub route_color: Option<String>,
    /// scheduled arrival time
    #[serde(rename = "scheduledArrival")]
    pub scheduled_arrival: String,
    /// scheduled departure time
    #[serde(rename = "scheduledDeparture")]
    pub scheduled_departure: String,
    pub to: crate::models::Place,
    pub trips: Vec<crate::models::TripInfo>,
}
