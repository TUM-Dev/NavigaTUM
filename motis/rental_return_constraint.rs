use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RentalReturnConstraint {
    #[default]
    None,
    AnyStation,
    RoundtripStation,
}
