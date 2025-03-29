use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leg {
    pub agency_id: Option<String>,
    pub agency_name: Option<String>,
    pub agency_url: Option<String>,
    /// For non-transit legs the distance traveled while traversing this leg in meters.
    pub distance: Option<f64>,
    /// Leg duration in seconds
    ///
    /// If leg is footpath:
    /// The footpath duration is derived from the default footpath duration.
    /// The query parameters `transferTimeFactor` and  `additionalTransferTime` are used as follows:
    /// - `leg.duration = defaultDuration * transferTimeFactor + additionalTransferTime.`
    ///
    /// In case the `defaultDuration` is needed, it can be calculated by
    /// - `defaultDuration = (leg.duration - additionalTransferTime) / transferTimeFactor`.
    ///
    /// Note that the default (if not explicitly provided in the query) values are
    /// - `transferTimeFactor = 1` and
    /// - `additionalTransferTime = 0`
    pub duration: i64,
    /// leg arrival time
    pub end_time: String,
    pub from: crate::models::Place,
    /// For transit legs, the headsign of the bus or train being used.
    /// [`None`] for non-transit legs.
    pub headsign: Option<String>,
    /// For transit legs, if the rider should stay on the vehicle as it changes route names.
    pub interline_with_previous_leg: Option<bool>,
    /// For transit legs, intermediate stops between the Place where the leg originates
    /// and the Place where the leg ends.
    ///
    /// [`None`] for non-transit legs.
    pub intermediate_stops: Option<Vec<crate::models::Place>>,
    pub leg_geometry: crate::models::EncodedPolyline,
    /// Transport mode for this leg
    pub mode: crate::models::Mode,
    /// Whether there is real-time data about this leg
    pub real_time: bool,
    pub rental: Option<crate::models::Rental>,
    pub route_color: Option<String>,
    pub route_short_name: Option<String>,
    pub route_text_color: Option<String>,
    pub route_type: Option<String>,
    /// scheduled leg arrival time
    pub scheduled_end_time: String,
    /// scheduled leg departure time
    pub scheduled_start_time: String,
    /// Filename and line number where this trip is from
    pub source: Option<String>,
    /// leg departure time
    pub start_time: String,
    /// A series of turn by turn instructions
    /// used for walking, biking and driving.
    pub steps: Option<Vec<crate::models::StepInstruction>>,
    pub to: crate::models::Place,
    pub trip_id: Option<String>,
}
