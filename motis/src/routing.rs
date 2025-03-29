/// Computes optimal connections from one place to another
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimalConnectionOptions {
    from_place: PlaceID,
    to_place: PlaceID,
    detailed_transfers: bool,
    // below only optional
    via: Option<Vec<StopID>>,
    via_minimum_stay: Option<Vec<u32>>,
    time: Option<chrono::NaiveDateTime>,
    max_transfers: Option<u32>,
    min_transfer_time: Option<u32>,
    additional_transfer_time: Option<u32>,
    transfer_time_factor: Option<u32>,
    max_matching_distance: Option<u32>,
    pedestrian_profile: Option<crate::models::PedestrianProfile>,
    use_routed_transfers: Option<bool>,
    transit_modes: Option<Vec<crate::models::Mode>>,
    direct_modes: Option<Vec<crate::models::Mode>>,
    pre_transit_modes: Option<Vec<crate::models::Mode>>,
    post_transit_modes: Option<Vec<crate::models::Mode>>,
    direct_rental_form_factors: Option<Vec<crate::models::RentalFormFactor>>,
    pre_transit_rental_form_factors: Option<Vec<crate::models::RentalFormFactor>>,
    post_transit_rental_form_factors: Option<Vec<crate::models::RentalFormFactor>>,
    direct_rental_propulsion_types: Option<Vec<crate::models::RentalPropulsionType>>,
    pre_transit_rental_propulsion_types: Option<Vec<crate::models::RentalPropulsionType>>,
    post_transit_rental_propulsion_types: Option<Vec<crate::models::RentalPropulsionType>>,
    direct_rental_providers: Option<Vec<String>>,
    pre_transit_rental_providers: Option<Vec<String>>,
    post_transit_rental_providers: Option<Vec<String>>,
    num_itineraries: Option<u32>,
    page_cursor: Option<String>,
    timetable_view: Option<bool>,
    arrive_by: Option<bool>,
    search_window: Option<u32>,
    require_bike_transport: Option<bool>,
    max_pre_transit_time: Option<u32>,
    max_post_transit_time: Option<u32>,
    max_direct_time: Option<u32>,
    fastest_direct_factor: Option<f32>,
    timeout: Option<u32>,
    passengers: Option<u32>,
    luggage: Option<u32>,
    with_fares: Option<bool>,
}

impl OptimalConnectionOptions {
    pub fn from_one_place_to_another(
        from_place: PlaceID,
        to_place: PlaceID,
        detailed_transfers: bool,
    ) -> Self {
        Self {
            from_place,
            to_place,
            detailed_transfers,
            via: None,
            via_minimum_stay: None,
            time: None,
            max_transfers: None,
            min_transfer_time: None,
            additional_transfer_time: None,
            transfer_time_factor: None,
            max_matching_distance: None,
            pedestrian_profile: None,
            use_routed_transfers: None,
            transit_modes: None,
            direct_modes: None,
            pre_transit_modes: None,
            post_transit_modes: None,
            direct_rental_form_factors: None,
            pre_transit_rental_form_factors: None,
            post_transit_rental_form_factors: None,
            direct_rental_propulsion_types: None,
            pre_transit_rental_propulsion_types: None,
            post_transit_rental_propulsion_types: None,
            direct_rental_providers: None,
            pre_transit_rental_providers: None,
            post_transit_rental_providers: None,
            num_itineraries: None,
            page_cursor: None,
            timetable_view: None,
            arrive_by: None,
            search_window: None,
            require_bike_transport: None,
            max_pre_transit_time: None,
            max_post_transit_time: None,
            max_direct_time: None,
            fastest_direct_factor: None,
            timeout: None,
            passengers: None,
            luggage: None,
            with_fares: None,
        }
    }
    ///List of via stops to visit
    ///
    /// See [`Self::via_minimum_stay`] to set a set a minimum stay duration for each via stop
    pub fn via(mut self, value: impl Iterator<Item = StopID>) -> Self {
        self.via = Some(value.into_iter().collect());
        self
    }
    /// For each via stop a minimum stay duration in minutes.
    ///
    /// If not set, the default is 0,0 - no stay required.
    ///
    /// The value 0 signals that it's allowed to stay in the same trip.
    /// This enables via stays without counting a transfer and can lead to better connections with less transfers.
    /// Transfer connections can still be found with `.via_minimum_stay([0])`.
    pub fn via_minimum_stay(mut self, value: impl Iterator<Item = StopID>) -> Self {
        self.via = Some(value.into_iter().collect());
        self
    }
    /// Departure/arrival time
    ///
    /// Defaults to the current time.
    /// See [`Self::arrive_by`] to switch if this is a arrival or departure time.
    pub fn time(mut self, value: impl Into<chrono::NaiveDateTime>) -> Self {
        self.time = Some(value.into());
        self
    }
    /// The times set via [`Self::time`], [`Self::search_window`] or included if `Self::timetable_view` is set refer to ***arrivals***
    ///
    /// Default:
    pub fn times_refer_to_arrivals(mut self) -> Self {
        self.arrive_by = Some(true);
        self
    }
    /// The times set via [`Self::time`], [`Self::search_window`] or included if `Self::timetable_view` is set refer to ***departures***
    ///
    /// Default:
    pub fn times_refer_to_departures(mut self) -> Self {
        self.arrive_by = Some(false);
        self
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaceID(String);
impl PlaceID {
    /// globally unique id of a stop
    pub fn from_stop_id(stop_id: impl ToString) -> Self {
        Self(stop_id.to_string())
    }
    /// Identifies a place by its lat/lon corrdiante and the [`level`-tag](http://wiki.openstreetmap.org/wiki/Key:level)
    pub fn from_coordinate_and_level(lat: f32, lon: f32, level: i8) -> Self {
        Self(format!("{lat},{lon},{level}"))
    }
}
impl From<StopID> for PlaceID {
    fn from(value: StopID) -> Self {
        Self(value.0)
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct StopID(String);
impl StopID {
    /// globally unique id of a stop
    pub fn from_stop_id(stop_id: impl ToString) -> Self {
        Self(stop_id.to_string())
    }
}
