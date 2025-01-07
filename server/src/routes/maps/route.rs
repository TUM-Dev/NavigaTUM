use crate::localisation;
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use serde_json::json;
use sqlx::PgPool;
use std::ops::Deref;
use tracing::{debug, error};
use valhalla_client::costing::{
    bicycle::BicycleType, pedestrian::PedestrianType, BicycleCostingOptions, Costing,
    MultimodalCostingOptions, PedestrianCostingOptions,
};
use valhalla_client::route::{
    Leg, Maneuver, ManeuverType, ShapePoint, Summary, TransitInfo, TransitStop, TransitStopType,
    TravelMode, Trip,
};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, utoipa::ToSchema)]
struct Coordinate {
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
}
impl From<ShapePoint> for Coordinate {
    fn from(value: ShapePoint) -> Self {
        Coordinate {
            lon: value.lon,
            lat: value.lat,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, utoipa::ToSchema)]
#[serde(untagged)]
enum RequestedLocation {
    /// Either an
    /// - external address which was looked up or
    /// - the users current location  
    Coordinate(Coordinate),
    /// Our (uni internal) key for location identification
    Location(String),
}
impl RequestedLocation {
    async fn try_resolve_coordinates(&self, pool: &PgPool) -> anyhow::Result<Option<Coordinate>> {
        match self {
            RequestedLocation::Coordinate(coords) => Ok(Some(*coords)),
            RequestedLocation::Location(key) => {
                let coords = sqlx::query_as!(
                    Coordinate,
                    r#"SELECT lat,lon
                    FROM de
                    WHERE key = $1 and
                          lat IS NOT NULL and
                          lon IS NOT NULL"#,
                    key
                )
                .fetch_optional(pool)
                .await?;
                Ok(coords)
            }
        }
    }
}

/// Transport mode the user wants to use
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum CostingRequest {
    Pedestrian,
    Bicycle,
    Motorcycle,
    Car,
    PublicTransit,
}
impl From<&RoutingRequest> for Costing {
    fn from(
        RoutingRequest {
            route_costing,
            pedestrian_type,
            ptw_type,
            bicycle_type,
            ..
        }: &RoutingRequest,
    ) -> Self {
        match route_costing {
            CostingRequest::Pedestrian => Costing::Pedestrian(
                PedestrianCostingOptions::builder().r#type(PedestrianType::from(*pedestrian_type)),
            ),
            CostingRequest::Bicycle => Costing::Bicycle(
                BicycleCostingOptions::builder().bicycle_type(BicycleType::from(*bicycle_type)),
            ),
            CostingRequest::Motorcycle => match ptw_type {
                PoweredTwoWheeledRestrictionRequest::Moped => {
                    Costing::Motorcycle(Default::default())
                }
                PoweredTwoWheeledRestrictionRequest::Motorcycle => {
                    Costing::MotorScooter(Default::default())
                }
            },
            CostingRequest::Car => Costing::Auto(Default::default()),
            CostingRequest::PublicTransit => {
                let pedestrian_costing = PedestrianCostingOptions::builder()
                    .r#type(PedestrianType::from(*pedestrian_type));
                Costing::Multimodal(
                    MultimodalCostingOptions::builder()
                        .pedestrian(pedestrian_costing)
                        .transit(Default::default()),
                )
            }
        }
    }
}

#[derive(Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
struct RoutingRequest {
    #[serde(flatten, default)]
    lang: localisation::LangQueryArgs,
    /// Start of the route
    from: RequestedLocation,
    /// Destination of the route
    to: RequestedLocation,
    /// Transport mode the user wants to use
    route_costing: CostingRequest,
    /// Does the user have specific walking restrictions?
    #[serde(default)]
    pedestrian_type: PedestrianTypeRequest,
    /// Does the user prefer mopeds or motorcycles for powered two-wheeled (ptw)?
    #[serde(default)]
    ptw_type: PoweredTwoWheeledRestrictionRequest,
    /// Which kind of bicycle do you ride?
    #[serde(default)]
    bicycle_type: BicycleRestrictionRequest,
}

/// Does the user have specific walking restrictions?
#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
enum PedestrianTypeRequest {
    #[default]
    None,
    Blind,
    // TODO
    // Wheelchair,
}

impl From<PedestrianTypeRequest> for PedestrianType {
    fn from(value: PedestrianTypeRequest) -> Self {
        match value {
            PedestrianTypeRequest::None => PedestrianType::Blind,
            PedestrianTypeRequest::Blind => PedestrianType::Blind,
            // TODO
            // PedestrianTypeRequest::Wheelchair => PedestrianType::Wheelchair,
        }
    }
}

/// Which kind of bicycle do you ride?
#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
enum BicycleRestrictionRequest {
    /// Road-bike
    ///
    /// A road-style bicycle with narrow tires that is generally lightweight and designed for speed on paved surfaces.
    Road,
    /// Hybrid- or City-bike
    ///
    /// A bicycle made mostly for city riding or casual riding on roads and paths with good surfaces.
    #[default]
    Hybrid,
    /// Cross-bike
    ///
    /// A cyclo-cross bicycle, which is similar to a road bicycle but with wider tires suitable to rougher surfaces.
    Cross,
    /// Mountain-bike
    ///
    /// A mountain bicycle suitable for most surfaces but generally heavier and slower on paved surfaces.
    Mountain,
}
impl From<BicycleRestrictionRequest> for BicycleType {
    fn from(bicycle_type: BicycleRestrictionRequest) -> Self {
        match bicycle_type {
            BicycleRestrictionRequest::Road => BicycleType::Road,
            BicycleRestrictionRequest::Hybrid => BicycleType::Hybrid,
            BicycleRestrictionRequest::Cross => BicycleType::Cross,
            BicycleRestrictionRequest::Mountain => BicycleType::Mountain,
        }
    }
}
/// Does the user have a moped or motorcycle
#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
enum PoweredTwoWheeledRestrictionRequest {
    #[default]
    Motorcycle,
    Moped,
}

/// Routing requests
///
/// **API IS EXPERIMENTAL AND ACTIVELY SUBJECT TO CHANGE**
///
/// The user specifies using provided origin (`from`) and destination (`to`) locations and a transport mode (`route_costing`) to tune their routing between the two locations.
/// The costing is fine-tuned by the server side accordingly.
///
/// Internally, this endpoint relies on
/// - [Valhalla](https://github.com/valhalla/valhalla) for routing for route calculation
/// - our database to resolve ids.
///   
///   You will need to look the ids up via [`/api/search`](#tag/locations/operation/search_handler) beforehand.
///   **Note:** [`/api/search`](#tag/locations/operation/search_handler) does support both university internal routing and external addressing.
///
/// **In the future (i.e. public transit routing currently is not implemented)**, it will als rely on either
/// - [OpenTripPlanner2](https://www.opentripplanner.org/) or
/// - [Motis](https://github.com/motis-project/motis)
#[utoipa::path(
    tags=["maps"],
    params(RoutingRequest),
    responses(
        (status = 200, description = "**Routing solution**", body=RoutingResponse, content_type = "application/json"),
        (status = 404, description = "**Not found.** The requested location does not exist", body = String, content_type = "text/plain", example = "Not found"),
    )
)]
#[get("/api/maps/route")]
pub async fn route_handler(
    args: web::Query<RoutingRequest>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let from = args.from.try_resolve_coordinates(&data.pool).await;
    let to = args.to.try_resolve_coordinates(&data.pool).await;
    let (from, to) = match (from, to) {
        (Ok(Some(from)), Ok(Some(to))) => (from, to),
        (Ok(None), _) | (_, Ok(None)) => {
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not found");
        }
        (Err(e), _) | (_, Err(e)) => {
            error!(from=?args.from,to=?args.to,error = ?e,"could not resolve into coordinates");
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to resolve key");
        }
    };

    if args.route_costing == CostingRequest::PublicTransit {
        return HttpResponse::NotImplemented()
            .content_type("text/plain")
            .body("public transit routing is not yet implemented");
    }

    let routing = data
        .valhalla
        .route(
            (from.lat as f32, from.lon as f32),
            (to.lat as f32, to.lon as f32),
            Costing::from(args.deref()),
            args.lang.should_use_english(),
        )
        .await;
    let response = match routing {
        Ok(response) => response,
        Err(e) => {
            error!(error=?e,"error routing");
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Could not generate a route, please try again later");
        }
    };
    debug!(routing_solution=?response,"got routing solution");

    HttpResponse::Ok().json(RoutingResponse::from(response))
}
#[derive(Serialize, Debug, utoipa::ToSchema)]
struct RoutingResponse {
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
            length_meters: value.length,
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
            instruction: value.instruction,
            verbal_transition_alert_instruction: value.verbal_transition_alert_instruction,
            verbal_pre_transition_instruction: value.verbal_pre_transition_instruction,
            verbal_post_transition_instruction: value.verbal_post_transition_instruction,
            street_names: value.street_names,
            begin_street_names: value.begin_street_names,
            time_seconds: value.time,
            length_meters: value.length,
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
