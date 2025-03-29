use serde::{Deserialize, Serialize};

/// - `NORMAL` - latitude / longitude coordinate or address
/// - `BIKESHARE` - bike sharing station
/// - `TRANSIT` - transit stop
///
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VertexType {
    #[default]
    Normal,
    Bikeshare,
    Transit,
}
