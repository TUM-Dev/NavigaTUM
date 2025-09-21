use motis_openapi_progenitor::types::*;
use serde::Serialize;

#[derive(Serialize, Debug, utoipa::ToSchema)]
pub struct MotisRoutingResponse {
    #[cfg(debug_assertions)]
    #[schema(ignore)]
    ///debug statistics
    pub debug_output: std::collections::HashMap<String, i64>,
    ///Direct trips by `WALK`, `BIKE`, `CAR`, etc. without time-dependency.
    ///
    /// The starting time (`arriveBy=false`) / arrival time
    /// (`arriveBy=true`) is always the queried `time` parameter (set to
    /// "now" if not set). But all `direct` connections are meant
    /// to be independent of absolute times.
    pub direct: Vec<ItineraryResponse>,
    ///list of itineraries
    pub itineraries: Vec<ItineraryResponse>,
    ///Use the cursor to get the next page of results.
    ///
    ///Insert the cursor
    /// into the request and post it to get the next page.
    /// The next page is a set of itineraries departing AFTER the last
    /// itinerary in this result.
    pub next_page_cursor: String,
    ///Use the cursor to get the previous page of results. Insert the
    /// cursor into the request and post it to get the previous page.
    /// The previous page is a set of itineraries departing BEFORE the first
    /// itinerary in the result for a depart after search. When using the
    /// default sort order the previous set of itineraries is inserted
    /// before the current result.
    pub previous_page_cursor: String,
}
impl From<PlanResponse> for MotisRoutingResponse {
    fn from(value: PlanResponse) -> Self {
        MotisRoutingResponse {
            #[cfg(debug_assertions)]
            debug_output: value.debug_output,
            direct: value
                .direct
                .into_iter()
                .map(ItineraryResponse::from)
                .collect(),
            itineraries: value
                .itineraries
                .into_iter()
                .map(ItineraryResponse::from)
                .collect(),
            next_page_cursor: value.next_page_cursor,
            previous_page_cursor: value.previous_page_cursor,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
pub struct ItineraryResponse {
    ///journey duration in seconds
    pub duration: i64,
    ///journey arrival time
    pub end_time: chrono::DateTime<chrono::offset::Utc>,
    ///Journey legs
    pub legs: Vec<MotisLegResponse>,
    ///journey departure time
    pub start_time: chrono::DateTime<chrono::offset::Utc>,
    ///The number of transfers this trip has.
    pub transfer_count: i64,
}

impl From<Itinerary> for ItineraryResponse {
    fn from(value: Itinerary) -> Self {
        ItineraryResponse {
            duration: value.duration,
            end_time: value.end_time,
            legs: value.legs.into_iter().map(MotisLegResponse::from).collect(),
            start_time: value.start_time,
            transfer_count: value.transfers,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
pub struct MotisLegResponse {
    /// Identifies a transit brand which is often synonymous with a transit agency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agency_id: Option<String>,
    /// Full name of the transit agency
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agency_name: Option<String>,
    /// URL of the transit agency
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agency_url: Option<String>,

    ///Alerts for this stop.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alerts: Vec<AlertResponse>,
    ///Whether this trip is cancelled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
    /// Distance in meters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    ///Leg duration in seconds
    ///
    ///If leg is footpath:
    ///  The footpath duration is derived from the default footpath
    ///  duration using the query parameters `transferTimeFactor` and
    ///  `additionalTransferTime` as follows:
    ///  `leg.duration = defaultDuration * transferTimeFactor +
    /// additionalTransferTime.`  In case the defaultDuration is
    /// needed, it can be calculated by  `defaultDuration =
    /// (leg.duration - additionalTransferTime) / transferTimeFactor`.
    ///  Note that the default values are `transferTimeFactor = 1` and
    ///  `additionalTransferTime = 0` in case they are not explicitly
    ///  provided in the query.
    pub duration: i64,
    ///leg arrival time
    pub end_time: chrono::DateTime<chrono::offset::Utc>,
    pub from: PlaceResponse,
    pub to: PlaceResponse,

    ///For transit legs, the headsign of the bus or train being used.
    ///For non-transit legs, null
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headsign: Option<String>,
    ///For transit legs, if the rider should stay on the vehicle as it
    /// changes route names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interline_with_previous_leg: Option<bool>,
    ///For transit legs, intermediate stops between the Place where the leg
    /// originates and the Place where the leg ends. For non-transit
    /// legs, null.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intermediate_stops: Vec<PlaceResponse>,
    ///A series of turn by turn instructions
    ///used for walking, biking and driving.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<StepInstructionResponse>,
    /// Polyline geometry (precision 6) of the leg.
    pub leg_geometry: String,
    pub mode: ModeResponse,

    ///Whether there is real-time data about this leg
    pub real_time: bool,
    ///Whether this leg was originally scheduled to run or is an additional
    /// service.
    /// Scheduled times will equal realtime times in this case.
    pub scheduled: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rental: Option<rental::RentalResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Route color designation that matches public facing material.
    ///
    /// Implementations should default to white (FFFFFF) when omitted or left empty.
    /// The color difference between `route_color` and `route_text_color` should provide sufficient contrast when viewed on a black and white screen.
    pub route_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_short_name: Option<String>,
    /// Legible color to use for text drawn against a background of `route_color`.
    ///
    /// Implementations should default to black (000000) when omitted or left empty.
    /// The color difference between `route_color` and `route_text_color` should provide sufficient contrast when viewed on a black and white screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_text_color: Option<String>,
    /// Indicates the type of transportation used on a route.
    ///
    /// According to <https://gtfs.org/reference/static/#routestxt> `route_type` Valid options are:
    ///
    /// -  0: Tram, Streetcar, Light rail. Any light rail or street level system within a metropolitan area.
    /// -  1: Subway, Metro. Any underground rail system within a metropolitan area.
    /// -  2: Rail. Used for intercity or long-distance travel.
    /// -  3: Bus. Used for short- and long-distance bus routes.
    /// -  4: Ferry. Used for short- and long-distance boat service.
    /// -  5: Cable tram. Used for street-level rail cars where the cable runs beneath the vehicle (e.g., cable car in San Francisco).
    /// -  6: Aerial lift, suspended cable car (e.g., gondola lift, aerial tramway). Cable transport where cabins, cars, gondolas or open chairs are suspended by means of one or more cables.
    /// -  7: Funicular. Any rail system designed for steep inclines.
    /// - 11: Trolleybus. Electric buses that draw power from overhead wires using poles.
    /// - 12: Monorail. Railway in which the track consists of a single rail or a beam.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_type: Option<i64>,

    ///scheduled leg arrival time
    pub scheduled_end_time: chrono::DateTime<chrono::offset::Utc>,
    ///scheduled leg departure time
    pub scheduled_start_time: chrono::DateTime<chrono::offset::Utc>,
    ///Filename and line number where this trip is from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    ///leg departure time
    pub start_time: chrono::DateTime<chrono::offset::Utc>,

    /// Identifies a trip
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<String>,
}

impl From<Leg> for MotisLegResponse {
    fn from(value: Leg) -> Self {
        assert_eq!(value.leg_geometry.precision, 6);
        MotisLegResponse {
            agency_id: value.agency_id,
            agency_name: value.agency_name,
            agency_url: value.agency_url,
            alerts: value.alerts.into_iter().map(AlertResponse::from).collect(),
            cancelled: value.cancelled,
            distance: value.distance,
            duration: value.duration,
            end_time: value.end_time,
            from: PlaceResponse::from(value.from),
            to: PlaceResponse::from(value.to),
            headsign: value.headsign,
            interline_with_previous_leg: value.interline_with_previous_leg,
            intermediate_stops: value
                .intermediate_stops
                .into_iter()
                .map(PlaceResponse::from)
                .collect(),
            steps: value
                .steps
                .into_iter()
                .map(StepInstructionResponse::from)
                .collect(),
            leg_geometry: value.leg_geometry.points,
            mode: ModeResponse::from(value.mode),
            real_time: value.real_time,
            scheduled: value.scheduled,
            rental: value.rental.map(rental::RentalResponse::from),
            route_color: value.route_color,
            route_short_name: value.route_short_name,
            route_text_color: value.route_text_color,
            route_type: value.route_type,
            scheduled_end_time: value.scheduled_end_time,
            scheduled_start_time: value.scheduled_start_time,
            source: value.source,
            start_time: value.start_time,
            trip_id: value.trip_id,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModeResponse {
    Walk,
    Bike,
    Rental,
    Car,
    CarParking,
    CarDropoff,
    Odm,
    Flex,
    Transit,
    Tram,
    Subway,
    Ferry,
    Airplane,
    Metro,
    Bus,
    Coach,
    Rail,
    HighspeedRail,
    LongDistance,
    NightRail,
    RegionalFastRail,
    RegionalRail,
    CableCar,
    Funicular,
    ArealLift,
    Other,
}
impl From<Mode> for ModeResponse {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Walk => ModeResponse::Walk,
            Mode::Bike => ModeResponse::Bike,
            Mode::Rental => ModeResponse::Rental,
            Mode::Car => ModeResponse::Car,
            Mode::CarParking => ModeResponse::CarParking,
            Mode::Odm => ModeResponse::Odm,
            Mode::Flex => ModeResponse::Flex,
            Mode::Transit => ModeResponse::Transit,
            Mode::Tram => ModeResponse::Tram,
            Mode::Subway => ModeResponse::Subway,
            Mode::Ferry => ModeResponse::Ferry,
            Mode::Airplane => ModeResponse::Airplane,
            Mode::Metro => ModeResponse::Metro,
            Mode::Bus => ModeResponse::Bus,
            Mode::Coach => ModeResponse::Coach,
            Mode::Rail => ModeResponse::Rail,
            Mode::HighspeedRail => ModeResponse::HighspeedRail,
            Mode::LongDistance => ModeResponse::LongDistance,
            Mode::NightRail => ModeResponse::NightRail,
            Mode::RegionalFastRail => ModeResponse::RegionalFastRail,
            Mode::RegionalRail => ModeResponse::RegionalRail,
            Mode::CableCar => ModeResponse::CableCar,
            Mode::Funicular => ModeResponse::Funicular,
            Mode::ArealLift => ModeResponse::ArealLift,
            Mode::Other => ModeResponse::Other,
            Mode::CarDropoff => ModeResponse::CarDropoff,
        }
    }
}

pub mod rental {
    use super::*;
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    pub struct RentalResponse {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub form_factor: Option<RentalFormFactorResponse>,
        ///Name of the station where the vehicle is picked up (empty for free
        /// floating vehicles)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub from_station_name: Option<String>,
        ///Rental URI for Android (deep link to the specific station or
        /// vehicle)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rental_uri_android: Option<String>,
        ///Rental URI for iOS (deep link to the specific station or vehicle)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rental_uri_ios: Option<String>,
        ///Rental URI for web (deep link to the specific station or vehicle)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub rental_uri_web: Option<String>,
        ///Name of the station
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub station_name: Option<String>,
        ///Vehicle share system ID
        pub system_id: String,
        ///Vehicle share system name
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub system_name: Option<String>,
        ///Name of the station where the vehicle is returned (empty for free
        /// floating vehicles)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub to_station_name: Option<String>,
        ///URL of the vehicle share system
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,
    }

    impl From<Rental> for RentalResponse {
        fn from(value: Rental) -> Self {
            RentalResponse {
                form_factor: value.form_factor.map(RentalFormFactorResponse::from),
                from_station_name: value.from_station_name,
                rental_uri_android: value.rental_uri_android,
                rental_uri_ios: value.rental_uri_ios,
                rental_uri_web: value.rental_uri_web,
                station_name: value.station_name,
                system_id: value.system_id,
                system_name: value.system_name,
                to_station_name: value.to_station_name,
                url: value.url,
            }
        }
    }
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum RentalFormFactorResponse {
        Bicycle,
        CargoBicycle,
        Car,
        Moped,
        ScooterStanding,
        ScooterSeated,
        Other,
    }
    impl From<RentalFormFactor> for RentalFormFactorResponse {
        fn from(value: RentalFormFactor) -> Self {
            match value {
                RentalFormFactor::Bicycle => RentalFormFactorResponse::Bicycle,
                RentalFormFactor::CargoBicycle => RentalFormFactorResponse::CargoBicycle,
                RentalFormFactor::Car => RentalFormFactorResponse::Car,
                RentalFormFactor::Moped => RentalFormFactorResponse::Moped,
                RentalFormFactor::ScooterStanding => RentalFormFactorResponse::ScooterStanding,
                RentalFormFactor::ScooterSeated => RentalFormFactorResponse::ScooterSeated,
                RentalFormFactor::Other => RentalFormFactorResponse::Other,
            }
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
pub struct StepInstructionResponse {
    /// Experimental. Indicates whether access to this part of the route is
    /// restricted.
    /// See: <https://wiki.openstreetmap.org/wiki/Conditional_restrictions>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_restriction: Option<String>,
    ///Not implemented!
    ///This step is on an open area, such as a plaza or train platform,
    ///and thus the directions should say something like "cross"
    pub area: bool,
    pub distance: f64,
    ///decline in meters across this path segment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_down: Option<i64>,
    ///incline in meters across this path segment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_up: Option<i64>,
    ///Not implemented!
    ///When exiting a highway or traffic circle, the exit name/number.
    pub exit: String,
    pub from_level: f64,
    ///OpenStreetMap way index
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub osm_way: Option<i64>,
    /// Polyline geometry (precision 6) of the leg.
    pub polyline: String,
    pub relative_direction: DirectionResponse,
    ///Indicates whether or not a street changes direction at an
    /// intersection.
    pub stay_on: bool,
    ///The name of the street.
    pub street_name: String,
    pub to_level: f64,
    ///Indicates that a fee must be paid by general traffic to use a road,
    /// road bridge or road tunnel.
    pub toll: Option<bool>,
}
impl From<StepInstruction> for StepInstructionResponse {
    fn from(value: StepInstruction) -> Self {
        StepInstructionResponse {
            access_restriction: value.access_restriction,
            area: value.area,
            distance: value.distance,
            elevation_down: value.elevation_down,
            elevation_up: value.elevation_up,
            exit: value.exit,
            from_level: value.from_level,
            osm_way: value.osm_way,
            polyline: value.polyline.points,
            relative_direction: DirectionResponse::from(value.relative_direction),
            stay_on: value.stay_on,
            street_name: value.street_name,
            to_level: value.to_level,
            toll: value.toll,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DirectionResponse {
    Depart,
    HardLeft,
    Left,
    SlightlyLeft,
    Continue,
    SlightlyRight,
    Right,
    HardRight,
    CircleClockwise,
    CircleCounterclockwise,
    Stairs,
    Elevator,
    UturnLeft,
    UturnRight,
}
impl From<Direction> for DirectionResponse {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Depart => DirectionResponse::Depart,
            Direction::HardLeft => DirectionResponse::HardLeft,
            Direction::Left => DirectionResponse::Left,
            Direction::SlightlyLeft => DirectionResponse::SlightlyLeft,
            Direction::Continue => DirectionResponse::Continue,
            Direction::SlightlyRight => DirectionResponse::SlightlyRight,
            Direction::Right => DirectionResponse::Right,
            Direction::HardRight => DirectionResponse::HardRight,
            Direction::CircleClockwise => DirectionResponse::CircleClockwise,
            Direction::CircleCounterclockwise => DirectionResponse::CircleCounterclockwise,
            Direction::Stairs => DirectionResponse::Stairs,
            Direction::Elevator => DirectionResponse::Elevator,
            Direction::UturnLeft => DirectionResponse::UturnLeft,
            Direction::UturnRight => DirectionResponse::UturnRight,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
pub struct PlaceResponse {
    ///Alerts for this stop.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alerts: Vec<AlertResponse>,
    ///arrival time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrival: Option<chrono::DateTime<chrono::offset::Utc>>,
    ///scheduled arrival time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_arrival: Option<chrono::DateTime<chrono::offset::Utc>>,
    ///scheduled departure time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_departure: Option<chrono::DateTime<chrono::offset::Utc>>,
    ///Whether this stop is cancelled due to the realtime situation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
    ///departure time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub departure: Option<chrono::DateTime<chrono::offset::Utc>>,

    pub lat: f64,
    pub level: f64,
    pub lon: f64,

    ///name of the transit stop / PoI / address
    pub name: String,
    ///description of the location that provides more detailed information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// scheduled track from the static schedule timetable dataset
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_track: Option<String>,
    ///The ID of the stop. This is often something that users don't care
    /// about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_id: Option<String>,
    ///The current track/platform information, updated with real-time
    /// updates if available. Can be missing if neither real-time
    /// updates nor the schedule timetable contains track information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertex_type: Option<VertexTypeResponse>,
}

impl From<Place> for PlaceResponse {
    fn from(value: Place) -> Self {
        PlaceResponse {
            alerts: value.alerts.into_iter().map(AlertResponse::from).collect(),
            arrival: value.arrival,
            cancelled: value.cancelled,
            departure: value.departure,
            lat: value.lat,
            level: value.level,
            lon: value.lon,
            name: value.name,
            description: value.description,
            scheduled_arrival: value.scheduled_arrival,
            scheduled_departure: value.scheduled_departure,
            scheduled_track: value.scheduled_track,
            stop_id: value.stop_id,
            track: value.track,
            vertex_type: value.vertex_type.map(VertexTypeResponse::from),
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VertexTypeResponse {
    ///  latitude / longitude coordinate or address
    Normal,
    /// bike sharing station
    Bikeshare,
    /// transit stop
    Transit,
}
impl From<VertexType> for VertexTypeResponse {
    fn from(value: VertexType) -> Self {
        match value {
            VertexType::Normal => VertexTypeResponse::Normal,
            VertexType::Bikeshare => VertexTypeResponse::Bikeshare,
            VertexType::Transit => VertexTypeResponse::Transit,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
pub struct AlertResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<AlertCauseResponse>,
    ///Description of the cause of the alert that allows for
    /// agency-specific language; more specific than the Cause.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause_detail: Option<String>,
    ///Description for the alert.
    ///This plain-text string will be formatted as the body of the alert
    /// (or shown on an explicit "expand" request by the user).
    /// The information in the description should add to the information of
    /// the header.
    pub description_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<AlertEffectResponse>,
    ///Description of the effect of the alert that allows for
    /// agency-specific language; more specific than the Effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_detail: Option<String>,
    ///Header for the alert. This plain-text string will be highlighted,
    /// for example in boldface.
    pub header_text: String,
    ///Text describing the appearance of the linked image in the image
    /// field (e.g., in case the image can't be displayed or the
    /// user can't see the image for accessibility reasons). See the
    /// HTML spec for alt image text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_alternative_text: Option<String>,
    ///IANA media type as to specify the type of image to be displayed. The
    /// type must start with "image/"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_media_type: Option<String>,
    ///String containing an URL linking to an image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_level: Option<AlertSeverityLevelResponse>,
    ///The URL which provides additional information about the alert.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
impl From<Alert> for AlertResponse {
    fn from(value: Alert) -> Self {
        AlertResponse {
            cause: value.cause.map(AlertCauseResponse::from),
            cause_detail: value.cause_detail,
            description_text: value.description_text,
            effect: value.effect.map(AlertEffectResponse::from),
            effect_detail: value.effect_detail,
            header_text: value.header_text,
            image_alternative_text: value.image_alternative_text,
            image_media_type: value.image_media_type,
            image_url: value.image_url,
            severity_level: value.severity_level.map(AlertSeverityLevelResponse::from),
            url: value.url,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertCauseResponse {
    UnknownCause,
    OtherCause,
    TechnicalProblem,
    Strike,
    Demonstration,
    Accident,
    Holiday,
    Weather,
    Maintenance,
    Construction,
    PoliceActivity,
    MedicalEmergency,
}
impl From<AlertCause> for AlertCauseResponse {
    fn from(value: AlertCause) -> Self {
        match value {
            AlertCause::UnknownCause => AlertCauseResponse::UnknownCause,
            AlertCause::OtherCause => AlertCauseResponse::OtherCause,
            AlertCause::TechnicalProblem => AlertCauseResponse::TechnicalProblem,
            AlertCause::Strike => AlertCauseResponse::Strike,
            AlertCause::Demonstration => AlertCauseResponse::Demonstration,
            AlertCause::Accident => AlertCauseResponse::Accident,
            AlertCause::Holiday => AlertCauseResponse::Holiday,
            AlertCause::Weather => AlertCauseResponse::Weather,
            AlertCause::Maintenance => AlertCauseResponse::Maintenance,
            AlertCause::Construction => AlertCauseResponse::Construction,
            AlertCause::PoliceActivity => AlertCauseResponse::PoliceActivity,
            AlertCause::MedicalEmergency => AlertCauseResponse::MedicalEmergency,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertEffectResponse {
    NoService,
    ReducedService,
    SignificantDelays,
    Detour,
    AdditionalService,
    ModifiedService,
    OtherEffect,
    UnknownEffect,
    StopMoved,
    NoEffect,
    AccessibilityIssue,
}
impl From<AlertEffect> for AlertEffectResponse {
    fn from(value: AlertEffect) -> Self {
        match value {
            AlertEffect::NoService => AlertEffectResponse::NoService,
            AlertEffect::ReducedService => AlertEffectResponse::ReducedService,
            AlertEffect::SignificantDelays => AlertEffectResponse::SignificantDelays,
            AlertEffect::Detour => AlertEffectResponse::Detour,
            AlertEffect::AdditionalService => AlertEffectResponse::AdditionalService,
            AlertEffect::ModifiedService => AlertEffectResponse::ModifiedService,
            AlertEffect::OtherEffect => AlertEffectResponse::OtherEffect,
            AlertEffect::UnknownEffect => AlertEffectResponse::UnknownEffect,
            AlertEffect::StopMoved => AlertEffectResponse::StopMoved,
            AlertEffect::NoEffect => AlertEffectResponse::NoEffect,
            AlertEffect::AccessibilityIssue => AlertEffectResponse::AccessibilityIssue,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverityLevelResponse {
    Unknown,
    Info,
    Warning,
    Severe,
}

impl From<AlertSeverityLevel> for AlertSeverityLevelResponse {
    fn from(value: AlertSeverityLevel) -> Self {
        match value {
            AlertSeverityLevel::Info => AlertSeverityLevelResponse::Info,
            AlertSeverityLevel::Warning => AlertSeverityLevelResponse::Warning,
            AlertSeverityLevel::Severe => AlertSeverityLevelResponse::Severe,
            AlertSeverityLevel::UnknownSeverity => AlertSeverityLevelResponse::Unknown,
        }
    }
}
