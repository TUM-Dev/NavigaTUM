use serde::{Deserialize, Serialize};

/// # Street modes
///
///   - `WALK`
///   - `BIKE`
///   - `RENTAL`
///   - `CAR`
///   - `CAR_PARKING`
///
/// # Transit modes
///
///   - `TRANSIT`:
///   - `TRAM`:
///
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Mode {
    #[default]
    Walk,
    Bike,
    /// Experimental. Expect unannounced breaking changes (without version bumps).
    Rental,
    Car,
    CarParking,
    /// translates to:
    /// - [`Self::Rail`]
    /// - [`Self::HighspeedRail`]
    /// - [`Self::Subway`]
    /// - [`Self::Tram`]
    /// - [`Self::Bus`]
    /// - [`Self::Ferry`]
    /// - [`Self::Airplane`]
    /// - [`Self::Coach`]
    Transit,
    /// trams
    Tram,
    /// subway trains
    Subway,
    /// ferries
    Ferry,
    /// airline flights
    Airplane,
    /// metro trains
    Metro,
    /// short distance buses (does not include [`Self::Coach`])
    Bus,
    /// long distance buses (does not include [`Self::Bus`])
    Coach,
    /// translates to:
    /// - [`Self::HighspeedRail`]
    /// - [`Self::LongDistance`]
    /// - [`Self::NightRail`]
    /// - [`Self::RegionalRail`]
    /// - [`Self::RegionalFastRail`]
    Rail,
    /// long distance high speed trains (e.g. TGV)
    HighspeedRail,
    /// long distance inter city trains
    LongDistance,
    /// long distance night trains
    NightRail,
    /// regional express routes that skip low traffic stops to be faster
    RegionalFastRail,
    /// regional train
    RegionalRail,
    Other,
}
