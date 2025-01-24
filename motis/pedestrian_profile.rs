use serde::{Deserialize, Serialize};

/// Different accessibility profiles for pedestrians.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PedestrianProfile {
    #[default]
    Foot,
    Wheelchair,
}
