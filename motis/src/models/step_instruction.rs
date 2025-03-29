use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepInstruction {
    /// Not implemented!
    /// This step is on an open area, such as a plaza or train platform,
    /// and thus the directions should say something like "cross"
    pub area: bool,
    /// The distance in meters that this step takes.
    pub distance: f64,
    /// Not implemented!
    /// When exiting a highway or traffic circle, the exit name/number.
    pub exit: String,
    /// level where this segment starts, based on OpenStreetMap data
    pub from_level: f64,
    /// OpenStreetMap way index
    pub osm_way: Option<i64>,
    pub polyline: crate::models::EncodedPolyline,
    pub relative_direction: crate::models::Direction,
    /// Not implemented!
    ///
    /// Indicates whether or not a street changes direction at an intersection.
    pub stay_on: bool,
    /// The name of the street.
    pub street_name: String,
    /// level where this segment starts, based on OpenStreetMap data
    pub to_level: f64,
}
