use serde::{Deserialize, Serialize};

/// footpath from one location to another
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Footpath {
    /// footpath duration in minutes according to GTFS (+heuristics)
    ///
    /// [`None`] if the GTFS did not contain a footpath
    pub default: Option<f64>,
    /// footpath duration in minutes for the foot profile
    ///
    /// [`None`] if no path was found with the foot profile
    pub foot: Option<f64>,
    pub to: crate::models::Place,
    /// footpath duration in minutes for the wheelchair profile
    ///
    /// [`None`] if no path was found with the wheelchair profile
    pub wheelchair: Option<f64>,
    /// true if the wheelchair path uses an elevator
    ///
    /// [`None`] if no path was found with the wheelchair profile
    pub wheelchair_uses_elevator: Option<bool>,
}
