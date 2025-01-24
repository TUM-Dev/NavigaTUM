use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RentalPropulsionType {
    #[default]
    Human,
    ElectricAssist,
    Electric,
    Combustion,
    CombustionDiesel,
    Hybrid,
    PlugInHybrid,
    HydrogenFuelCell,
}
