use serde::Serialize;
use valhalla_client::route::{
    Leg, Maneuver, ManeuverType, Summary, TransitInfo, TransitStop, TransitStopType, TravelMode,
    Trip,
};

use crate::routes::maps::route::Coordinate;

#[derive(Serialize, Debug, utoipa::ToSchema)]
pub struct RoutingResponse {
    /// A trip contains one (or more) legs.
    ///
    /// A leg is created when routing stops, which currently only happens at the ends (`from`, `to`).
    #[schema(min_items = 1, max_items = 1)]
    legs: Vec<LegResponse>,
    /// Trip summary
    summary: SummaryResponse,
}
impl From<Trip> for RoutingResponse {
    fn from(value: Trip) -> Self {
        RoutingResponse {
            legs: value.legs.into_iter().map(LegResponse::from).collect(),
            summary: SummaryResponse::from(value.summary),
        }
    }
}
#[derive(Serialize, Debug, utoipa::ToSchema)]
struct SummaryResponse {
    /// Estimated elapsed time in seconds
    #[schema(example = 201.025)]
    time_seconds: f64,
    /// Distance traveled in meters
    #[schema(example = 103.01)]
    length_meters: f64,
    /// If the path uses one or more toll segments
    has_toll: bool,
    /// If the path uses one or more highway segments
    has_highway: bool,
    ///  if the path uses one or more ferry segments
    has_ferry: bool,
    /// Minimum latitude of the sections bounding box
    #[schema(example = 48.26244490906312)]
    min_lat: f64,
    /// Minimum longitude of the sections bounding box
    #[schema(example = 48.26244490906312)]
    min_lon: f64,
    /// Maximum latitude of the sections bounding box
    #[schema(example = 48.26244490906312)]
    max_lat: f64,
    /// Maximum longitude of the sections bounding box
    #[schema(example = 48.26244490906312)]
    max_lon: f64,
}
impl From<Summary> for SummaryResponse {
    fn from(value: Summary) -> Self {
        SummaryResponse {
            time_seconds: value.time,
            length_meters: value.length * 1000.0,
            has_toll: value.has_toll,
            has_highway: value.has_highway,
            has_ferry: value.has_ferry,
            min_lat: value.min_lat,
            min_lon: value.min_lon,
            max_lat: value.max_lat,
            max_lon: value.max_lon,
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
struct LegResponse {
    summary: SummaryResponse,
    maneuvers: Vec<ManeuverResponse>,
    shape: Vec<Coordinate>,
}
impl From<Leg> for LegResponse {
    fn from(value: Leg) -> Self {
        LegResponse {
            summary: SummaryResponse::from(value.summary),
            maneuvers: value
                .maneuvers
                .into_iter()
                .map(ManeuverResponse::from)
                .collect(),
            shape: value.shape.into_iter().map(Coordinate::from).collect(),
        }
    }
}
#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug, utoipa::ToSchema)]
struct ManeuverResponse {
    r#type: ManeuverTypeResponse,

    instruction: String,

    /// Text suitable for use as a verbal alert in a navigation application
    ///
    /// The transition alert instruction will prepare the user for the forthcoming transition
    #[schema(examples("Turn right onto North Prince Street"))]
    verbal_transition_alert_instruction: Option<String>,

    /// Text suitable for use as a verbal message immediately prior to the maneuver transition
    #[schema(examples("Turn right onto North Prince Street, U.S. 2 22"))]
    verbal_pre_transition_instruction: Option<String>,
    /// Text suitable for use as a verbal message immediately after the maneuver transition
    #[schema(examples("Continue on U.S. 2 22 for 3.9 miles"))]
    verbal_post_transition_instruction: Option<String>,
    /// List of street names that are consistent along the entire nonobvious maneuver
    #[schema(examples(json!(["Münchnerstraße"])))]
    street_names: Option<Vec<String>>,
    /// When present, these are the street names at the beginning (transition point) of the
    /// nonobvious maneuver (if they are different from the names that are consistent along the
    /// entire nonobvious maneuver)
    #[schema(examples(json!(["Josef Fischaber Straße"])))]
    begin_street_names: Option<Vec<String>>,
    /// Estimated time along the maneuver in seconds
    #[schema(example = 201.025)]
    time_seconds: f64,
    /// Maneuver length in meters
    #[schema(example = 103.01)]
    length_meters: f64,
    /// Index into the list of shape points for the start of the maneuver
    #[schema(example = 0)]
    begin_shape_index: usize,
    /// Index into the list of shape points for the end of the maneuver
    #[schema(example = 3)]
    end_shape_index: usize,
    /// `true` if a toll booth is encountered on this maneuver
    toll: Option<bool>,
    /// `true` if a highway is encountered on this maneuver
    highway: Option<bool>,
    /// `true` if the maneuver is unpaved or rough pavement, or has any portions that have rough
    /// pavement
    rough: Option<bool>,
    /// `true` if a gate is encountered on this maneuver
    gate: Option<bool>,
    /// `true` if a ferry is encountered on this maneuver
    ferry: Option<bool>,
    /// The spoke to exit roundabout after entering
    #[schema(example = 2)]
    roundabout_exit_count: Option<i64>,
    /// Written depart time instruction
    ///
    /// Typically used with a transit maneuver
    #[schema(examples("Depart: 8:04 AM from 8 St - NYU"))]
    depart_instruction: Option<String>,
    /// Text suitable for use as a verbal depart time instruction
    ///
    /// Typically used with a transit maneuver
    #[schema(examples("Depart at 8:04 AM from 8 St - NYU"))]
    verbal_depart_instruction: Option<String>,
    /// Written arrive time instruction
    ///
    /// Typically used with a transit maneuver
    #[schema(examples("Arrive: 8:10 AM at 34 St - Herald Sq"))]
    arrive_instruction: Option<String>,
    /// Text suitable for use as a verbal arrive time instruction
    ///
    /// Typically used with a transit maneuver
    #[schema(examples("Arrive at 8:10 AM at 34 St - Herald Sq"))]
    verbal_arrive_instruction: Option<String>,
    /// Contains the attributes that describe a specific transit route
    transit_info: Option<TransitInfoResponse>,
    /// `true` if `verbal_pre_transition_instruction` has been appended with
    /// the verbal instruction of the next maneuver
    verbal_multi_cue: Option<bool>,
    /// Travel mode
    #[schema(examples("drive", "pedestrian", "bicycle", "public_transit"))]
    travel_mode: TravelModeResponse,
}
impl From<Maneuver> for ManeuverResponse {
    fn from(value: Maneuver) -> Self {
        ManeuverResponse {
            r#type: ManeuverTypeResponse::from(value.type_),
            instruction: value
                .instruction
                .strip_suffix(".")
                .map(|s| s.to_string())
                .unwrap_or(value.instruction),
            verbal_transition_alert_instruction: value.verbal_transition_alert_instruction,
            verbal_pre_transition_instruction: value.verbal_pre_transition_instruction,
            verbal_post_transition_instruction: value.verbal_post_transition_instruction,
            street_names: value.street_names,
            begin_street_names: value.begin_street_names,
            time_seconds: value.time,
            length_meters: value.length * 1000.0,
            begin_shape_index: value.begin_shape_index,
            end_shape_index: value.end_shape_index,
            toll: value.toll,
            highway: value.highway,
            rough: value.rough,
            gate: value.gate,
            ferry: value.ferry,
            roundabout_exit_count: value.roundabout_exit_count,
            depart_instruction: value.depart_instruction,
            verbal_depart_instruction: value.verbal_depart_instruction,
            arrive_instruction: value.arrive_instruction,
            verbal_arrive_instruction: value.verbal_arrive_instruction,
            transit_info: value.transit_info.map(TransitInfoResponse::from),
            verbal_multi_cue: value.verbal_multi_cue,
            travel_mode: TravelModeResponse::from(value.travel_mode),
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum ManeuverTypeResponse {
    None,
    Start,
    StartRight,
    StartLeft,
    Destination,
    DestinationRight,
    DestinationLeft,
    Becomes,
    Continue,
    SlightRight,
    Right,
    SharpRight,
    UturnRight,
    UturnLeft,
    SharpLeft,
    Left,
    SlightLeft,
    RampStraight,
    RampRight,
    RampLeft,
    ExitRight,
    ExitLeft,
    StayStraight,
    StayRight,
    StayLeft,
    Merge,
    RoundaboutEnter,
    RoundaboutExit,
    FerryEnter,
    FerryExit,
    Transit,
    TransitTransfer,
    TransitRemainOn,
    TransitConnectionStart,
    TransitConnectionTransfer,
    TransitConnectionDestination,
    PostTransitConnectionDestination,
    MergeRight,
    MergeLeft,
    ElevatorEnter,
    StepsEnter,
    EscalatorEnter,
    BuildingEnter,
    BuildingExit,
}
impl From<ManeuverType> for ManeuverTypeResponse {
    fn from(value: ManeuverType) -> Self {
        match value {
            ManeuverType::None => Self::None,
            ManeuverType::Start => Self::Start,
            ManeuverType::StartRight => Self::StartRight,
            ManeuverType::StartLeft => Self::StartLeft,
            ManeuverType::Destination => Self::Destination,
            ManeuverType::DestinationRight => Self::DestinationRight,
            ManeuverType::DestinationLeft => Self::DestinationLeft,
            ManeuverType::Becomes => Self::Becomes,
            ManeuverType::Continue => Self::Continue,
            ManeuverType::SlightRight => Self::SlightRight,
            ManeuverType::Right => Self::Right,
            ManeuverType::SharpRight => Self::SharpRight,
            ManeuverType::UturnRight => Self::UturnRight,
            ManeuverType::UturnLeft => Self::UturnLeft,
            ManeuverType::SharpLeft => Self::SharpLeft,
            ManeuverType::Left => Self::Left,
            ManeuverType::SlightLeft => Self::SlightLeft,
            ManeuverType::RampStraight => Self::RampStraight,
            ManeuverType::RampRight => Self::RampRight,
            ManeuverType::RampLeft => Self::RampLeft,
            ManeuverType::ExitRight => Self::ExitRight,
            ManeuverType::ExitLeft => Self::ExitLeft,
            ManeuverType::StayStraight => Self::StayStraight,
            ManeuverType::StayRight => Self::StayRight,
            ManeuverType::StayLeft => Self::StayLeft,
            ManeuverType::Merge => Self::Merge,
            ManeuverType::RoundaboutEnter => Self::RoundaboutEnter,
            ManeuverType::RoundaboutExit => Self::RoundaboutExit,
            ManeuverType::FerryEnter => Self::FerryEnter,
            ManeuverType::FerryExit => Self::FerryExit,
            ManeuverType::Transit => Self::Transit,
            ManeuverType::TransitTransfer => Self::TransitTransfer,
            ManeuverType::TransitRemainOn => Self::TransitRemainOn,
            ManeuverType::TransitConnectionStart => Self::TransitConnectionStart,
            ManeuverType::TransitConnectionTransfer => Self::TransitConnectionTransfer,
            ManeuverType::TransitConnectionDestination => Self::TransitConnectionDestination,
            ManeuverType::PostTransitConnectionDestination => {
                Self::PostTransitConnectionDestination
            }
            ManeuverType::MergeRight => Self::MergeRight,
            ManeuverType::MergeLeft => Self::MergeLeft,
            ManeuverType::ElevatorEnter => Self::ElevatorEnter,
            ManeuverType::StepsEnter => Self::StepsEnter,
            ManeuverType::EscalatorEnter => Self::EscalatorEnter,
            ManeuverType::BuildingEnter => Self::BuildingEnter,
            ManeuverType::BuildingExit => Self::BuildingExit,
        }
    }
}
#[derive(Serialize, Debug, utoipa::ToSchema)]

struct TransitInfoResponse {
    /// Global transit route identifier
    ///
    /// **Tipp:** you use these as feed-ids in transitland.
    /// Example: <https://www.transit.land/feeds/f-9q9-bart>
    #[schema(examples("f-9q9-bart", "f-zeus~schwäbisch~gmünd~gbfs"))]
    onestop_id: String,
    /// Short name describing the transit route
    #[schema(examples("N"))]
    short_name: String,
    /// Long name describing the transit route
    #[schema(examples("Broadway Express"))]
    long_name: String,
    /// The sign on a public transport vehicle that identifies the route destination to passengers
    #[schema(examples("ASTORIA - DITMARS BLVD"))]
    headsign: String,
    /// The numeric color value associated with a transit route
    ///
    /// The value for yellow would be `16567306`
    #[schema(examples(16567306))]
    color: i32,
    /// The numeric text color value associated with a transit route
    ///
    /// The value for black would be `0`
    #[schema(examples(0))]
    text_color: String,
    /// The description of the transit route
    #[schema(examples(r#"Trains operate from Ditmars Boulevard, Queens, to Stillwell Avenue, Brooklyn, at all times
N trains in Manhattan operate along Broadway and across the Manhattan Bridge to and from Brooklyn.
Trains in Brooklyn operate along 4th Avenue, then through Borough Park to Gravesend.
Trains typically operate local in Queens, and either express or local in Manhattan and Brooklyn,
depending on the time. Late night trains operate via Whitehall Street, Manhattan.
Late night service is local"#))]
    description: String,
    /// Global operator/agency identifier
    ///
    /// **Tipp:** you use these as feed-ids in transitland.
    /// Example: <https://www.transit.land/feeds/o-u281z9-mvv>
    #[schema(examples("o-u281z9-mvv"))]
    operator_onestop_id: String,
    /// Operator/agency name
    ///
    /// Short name is used over long name
    #[schema(examples(
        "BART",
        "King County Marine Division",
        "Münchner Verkehrs- und Tarifverbund (MVV)"
    ))]
    operator_name: String,
    /// Operator/agency URL
    #[schema(examples("http://web.mta.info/", "http://www.mvv-muenchen.de/"))]
    operator_url: String,
    /// A list of the stops/stations associated with a specific transit route
    transit_stops: Vec<TransitStopResponse>,
}
impl From<TransitInfo> for TransitInfoResponse {
    fn from(value: TransitInfo) -> Self {
        TransitInfoResponse {
            onestop_id: value.onestop_id,
            short_name: value.short_name,
            long_name: value.long_name,
            headsign: value.headsign,
            color: value.color,
            text_color: value.text_color,
            description: value.description,
            operator_onestop_id: value.operator_onestop_id,
            operator_name: value.operator_name,
            operator_url: value.operator_url,
            transit_stops: value
                .transit_stops
                .into_iter()
                .map(TransitStopResponse::from)
                .collect(),
        }
    }
}
#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum TravelModeResponse {
    Drive,
    Pedestrian,
    Bicycle,
    PublicTransit,
}
impl From<TravelMode> for TravelModeResponse {
    fn from(value: TravelMode) -> Self {
        match value {
            TravelMode::Drive => Self::Drive,
            TravelMode::Pedestrian => Self::Pedestrian,
            TravelMode::Bicycle => Self::Bicycle,
            TravelMode::Transit => Self::PublicTransit,
        }
    }
}
#[derive(Serialize, Debug, utoipa::ToSchema)]
struct TransitStopResponse {
    r#type: TransitStopTypeResponse,
    /// Name of the stop or station
    #[schema(examples("14 St - Union Sq"))]
    name: String,
    /// Arrival date and time
    arrival_date_time: chrono::NaiveDateTime,
    /// Departure date and time
    departure_date_time: chrono::NaiveDateTime,
    /// `true` if this stop is a marked as a parent stop
    is_parent_stop: bool,
    /// `true` if the times are based on an assumed schedule because the actual schedule is not known
    assumed_schedule: bool,
    /// Latitude of the transit stop in degrees
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude of the transit stop in degrees
    #[schema(example = 48.26244490906312)]
    lon: f64,
}
impl From<TransitStop> for TransitStopResponse {
    fn from(value: TransitStop) -> Self {
        TransitStopResponse {
            r#type: TransitStopTypeResponse::from(value.type_),
            name: value.name,
            arrival_date_time: value.arrival_date_time,
            departure_date_time: value.departure_date_time,
            is_parent_stop: value.is_parent_stop,
            assumed_schedule: value.assumed_schedule,
            lat: value.lat,
            lon: value.lon,
        }
    }
}
#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum TransitStopTypeResponse {
    /// Simple stop
    Stop,
    /// Station
    Station,
}
impl From<TransitStopType> for TransitStopTypeResponse {
    fn from(value: TransitStopType) -> Self {
        match value {
            TransitStopType::Stop => Self::Stop,
            TransitStopType::Station => Self::Station,
        }
    }
}
