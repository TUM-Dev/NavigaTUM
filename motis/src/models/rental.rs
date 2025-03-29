use serde::{Deserialize, Serialize};

/// Vehicle rental
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rental {
    pub form_factor: Option<crate::models::RentalFormFactor>,
    pub propulsion_type: Option<crate::models::RentalPropulsionType>,
    /// Rental URI for Android (deep link to the specific station or vehicle)
    pub rental_uri_android: Option<String>,
    /// Rental URI for iOS (deep link to the specific station or vehicle)
    pub rental_uri_ios: Option<String>,
    /// Rental URI for web (deep link to the specific station or vehicle)
    pub rental_uri_web: Option<String>,
    pub return_constraint: Option<crate::models::RentalReturnConstraint>,
    /// Name of the station
    pub station_name: Option<String>,
    /// Vehicle share system ID
    pub system_id: String,
    /// Vehicle share system name
    pub system_name: Option<String>,
    /// URL of the vehicle share system
    pub url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RentalFormFactor {
    Bicycle,
    CargoBicycle,
    Car,
    Moped,
    ScooterStanding,
    ScooterSeated,
    Other,
}

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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RentalReturnConstraint {
    #[default]
    None,
    AnyStation,
    RoundtripStation,
}
