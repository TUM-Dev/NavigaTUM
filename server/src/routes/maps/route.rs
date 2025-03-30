use crate::localisation;
use actix_web::{HttpResponse, get, web};
use motis_openapi_progenitor::types as motis;
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
    BicycleCostingOptions, Costing, MultimodalCostingOptions, PedestrianCostingOptions,
    bicycle::BicycleType, pedestrian::PedestrianType,
};
use valhalla_client::route as valhalla;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, utoipa::ToSchema)]
struct Coordinate {
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
    /// [`level`-tag](http://wiki.openstreetmap.org/wiki/Key:level) the coordinate is at
    level: Option<f64>,
}
impl From<valhalla::ShapePoint> for Coordinate {
    fn from(value: valhalla::ShapePoint) -> Self {
        Coordinate {
            lon: value.lon,
            lat: value.lat,
            level: None,
        }
    }
}
impl From<geo_types::Point> for Coordinate {
    fn from(value: geo_types::Point) -> Self {
        Self {
            lat: value.y(),
            lon: value.x(),
            level: None,
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
/// The logical hierarchy of the response is:
/// - leg: The top level itinerary/options/alternatives presented to the user
/// - maneuver: A major vehicle/mobility option
/// - step: A (local) movement
///
/// Internally, this endpoint relies on
/// - [Valhalla](https://github.com/valhalla/valhalla) for routing for route calculation
/// - [Motis](https://github.com/motis-project/motis) for multi-criterial public transit routing
/// - our database to resolve ids.
///   
///   You will need to look the ids up via [`/api/search`](#tag/locations/operation/search_handler) beforehand.
///   **Note:** [`/api/search`](#tag/locations/operation/search_handler) does support both university internal routing and external addressing.
#[utoipa::path(
    tags=["maps"],
    params(RoutingRequest),
    responses(
        (status = 200, description = "**Routing solution**", body=RoutingResponse, content_type = "application/json"),
        (status = 404, description = "**Not found.** The requested location does not exist", body = String, content_type = "text/plain", example = "Not found"),
        (status = 500, description = "**Internal Server Error.** Could not generate a route, please try again later", body = String, content_type = "text/plain", example = "Could not generate a route, please try again later"),
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
        let routing = data
            .motis
            .plan(
                &format!(
                    "{},{},{}",
                    from.lat,
                    from.lon,
                    from.level.unwrap_or_default()
                ),
                &format!("{},{},{}", to.lat, to.lon, to.level.unwrap_or_default()),
            )
            .await;
        match routing {
            Ok(response) => {
                debug!(routing_solution=?response,"got routing solution");
                HttpResponse::Ok().json(RoutingResponse::from(response))
            }
            Err(e) => {
                error!(error=?e,"error routing");
                HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Could not generate a route, please try again later")
            }
        }
    } else {
        let routing = data
            .valhalla
            .route(
                (from.lat as f32, from.lon as f32),
                (to.lat as f32, to.lon as f32),
                Costing::from(args.deref()),
                args.lang.should_use_english(),
            )
            .await;
        match routing {
            Ok(response) => {
                debug!(routing_solution=?response,"got routing solution");
                HttpResponse::Ok().json(RoutingResponse::from(response))
            }
            Err(e) => {
                error!(error=?e,"error routing");
                HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Could not generate a route, please try again later")
            }
        }
    }
}
#[derive(Serialize, Debug, utoipa::ToSchema)]
struct RoutingResponse {
    /// A trip contains one (or more) legs.
    ///
    /// A leg can be thought of an itineary/option/alternative.
    legs: Vec<leg::LegResponse>,
    /// Overall summary over all legs
    ///
    /// Contrary to the legs, these summaries behave a bit different:
    /// - The times and lengths are the minimum of all options
    /// - the bounding boxes are the union of all legs
    summary: leg::SummaryResponse,
}
impl From<valhalla::Trip> for RoutingResponse {
    fn from(value: valhalla::Trip) -> Self {
        RoutingResponse {
            legs: value.legs.into_iter().map(leg::LegResponse::from).collect(),
            summary: leg::SummaryResponse::from(value.summary),
        }
    }
}
impl From<motis::PlanResponse> for RoutingResponse {
    fn from(value: motis::PlanResponse) -> Self {
        let summary = leg::SummaryResponse::from(value.direct.as_slice());
        Self {
            legs: value
                .itineraries
                .into_iter()
                .map(leg::LegResponse::from)
                .collect(),
            summary,
        }
    }
}

mod leg {
    use super::maneuver::*;
    use super::*;
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    pub(crate) struct LegResponse {
        /// Summary what happens in this leg
        summary: SummaryResponse,
        /// Maneuvers this leg contains
        ///
        /// A Maneuver can is equivalent to using a major vehicle option (train, bike, feet, ...).
        /// They contain steps which represent the fine-grained routing.
        maneuvers: Vec<ManeuverResponse>,
        /// The routes geometry
        shape: Vec<Coordinate>,
    }
    impl From<valhalla::Leg> for LegResponse {
        fn from(value: valhalla::Leg) -> Self {
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
    impl From<motis::Itinerary> for LegResponse {
        fn from(value: motis::Itinerary) -> Self {
            let summary = SummaryResponse::from(&value);
            let shape = value
                .legs
                .iter()
                .flat_map(|e| {
                    polyline::decode_polyline(&e.leg_geometry.points, 7)
                        .map(|l| l.into_points())
                        .unwrap_or_default()
                })
                .map(Coordinate::from)
                .collect();
            LegResponse {
                summary,
                maneuvers: value.legs.into_iter().map(ManeuverResponse::from).collect(),
                shape,
            }
        }
    }

    #[derive(Serialize, Debug, utoipa::ToSchema)]
    pub struct BBox {
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

    #[derive(Serialize, Debug, utoipa::ToSchema)]
    pub struct SummaryResponse {
        /// Estimated elapsed time in seconds
        #[schema(example = 201.025)]
        time_seconds: f64,
        /// Distance traveled in meters
        #[schema(example = 103.01)]
        length_meters: f64,
        /// A bounding box containing all items exactly
        bbox: BBox,
    }
    impl From<valhalla::Summary> for SummaryResponse {
        fn from(value: valhalla::Summary) -> Self {
            SummaryResponse {
                time_seconds: value.time,
                length_meters: value.length * 1000.0,
                min_lat: value.min_lat,
                min_lon: value.min_lon,
                max_lat: value.max_lat,
                max_lon: value.max_lon,
            }
        }
    }
    impl From<&[motis::Itinerary]> for SummaryResponse {
        fn from(value: &[motis::Itinerary]) -> Self {
            let summarys = value.iter().map(SummaryResponse::from).collect::<Vec<_>>();
            SummaryResponse {
                time_seconds: summarys.iter().map(|s| s.time_seconds).sum(),
                length_meters: summarys.iter().map(|s| s.length_meters).sum(),
                min_lat: summarys
                    .iter()
                    .map(|s| s.min_lat)
                    .min_by(f64::total_cmp)
                    .unwrap_or_default(),
                min_lon: summarys
                    .iter()
                    .map(|s| s.min_lon)
                    .min_by(f64::total_cmp)
                    .unwrap_or_default(),
                max_lat: summarys
                    .iter()
                    .map(|s| s.max_lat)
                    .max_by(f64::total_cmp)
                    .unwrap_or_default(),
                max_lon: summarys
                    .iter()
                    .map(|s| s.max_lon)
                    .max_by(f64::total_cmp)
                    .unwrap_or_default(),
            }
        }
    }
    impl From<&motis::Itinerary> for SummaryResponse {
        fn from(value: &motis::Itinerary) -> Self {
            let mut points = Vec::with_capacity(
                value
                    .legs
                    .iter()
                    .map(|l| l.leg_geometry.length as usize)
                    .sum(),
            );
            for leg in value.legs.iter() {
                let line_points = polyline::decode_polyline(&leg.leg_geometry.points, 7)
                    .map(|l| l.into_points())
                    .unwrap();
                for point in line_points {
                    points.push(point);
                }
            }
            SummaryResponse {
                time_seconds: value.duration as f64,
                length_meters: value
                    .legs
                    .iter()
                    .map(|s| s.distance.unwrap_or_default())
                    .sum(),
                min_lat: points
                    .iter()
                    .map(|p| p.0.x)
                    .min_by(f64::total_cmp)
                    .unwrap_or_default(),
                min_lon: points
                    .iter()
                    .map(|p| p.0.y)
                    .min_by(f64::total_cmp)
                    .unwrap_or_default(),
                max_lat: points
                    .iter()
                    .map(|p| p.0.x)
                    .max_by(f64::total_cmp)
                    .unwrap_or_default(),
                max_lon: points
                    .iter()
                    .map(|p| p.0.y)
                    .max_by(f64::total_cmp)
                    .unwrap_or_default(),
            }
        }
    }
}
mod maneuver {
    use super::step::*;
    use super::*;
    use crate::routes::maps::route::leg::BBox;
    use core::ops::Range;

    #[serde_with::skip_serializing_none]
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    pub struct ManeuverResponse {
        /// Travel mode
        #[schema(examples("drive", "pedestrian", "bicycle", "public_transit", "other"))]
        travel_mode: ManeuverTravelModeResponse,
        /// Summary what happens in this maneuver
        summary: ManeuverMetadataResponse,
        /// Contains attributes that describe a specific transit route
        transit_info: Option<TransitInfoResponse>,
        /// Steps contained in this maneuver
        ///
        /// Can be the equivalent of "walk down street" or "take this ICE"
        steps: StepResponse,
    }
    impl From<valhalla::Maneuver> for ManeuverResponse {
        fn from(value: valhalla::Maneuver) -> Self {
            ManeuverResponse {
                steps: todo!(),
                instruction: value
                    .instruction
                    .strip_suffix(".")
                    .map(|s| s.to_string())
                    .unwrap_or(value.instruction),
                verbal_transition_alert_instruction: value.verbal_transition_alert_instruction,
                verbal_pre_transition_instruction: value.verbal_pre_transition_instruction,
                verbal_post_transition_instruction: value.verbal_post_transition_instruction,
                time_seconds: value.time,
                length_meters: value.length * 1000.0,
                begin_shape_index: value.begin_shape_index,
                end_shape_index: value.end_shape_index,
                highway: value.highway,
                gate: value.gate,
                ferry: value.ferry,
                depart_instruction: value.depart_instruction,
                verbal_depart_instruction: value.verbal_depart_instruction,
                arrive_instruction: value.arrive_instruction,
                verbal_arrive_instruction: value.verbal_arrive_instruction,
                // we don't currently have transit info configured
                transit_info: None,
                travel_mode: TravelModeResponse::from(value.travel_mode),
            }
        }
    }

    impl From<motis::Leg> for ManeuverResponse {
        fn from(value: motis::Leg) -> Self {
            debug!(?value, "got leg");
            let transit_info = TransitInfoResponse {
                trip_id: value.trip_id,
                short_name: value.route_short_name,
                long_name: None,
                headsign: value.headsign,
                color: value.route_color,
                text_color: value.route_text_color,
                operator_id: value.agency_id,
                operator_name: value.agency_name,
                operator_url: value.agency_url,
                steps: value
                    .steps
                    .into_iter()
                    .map(TransitStepResponse::from)
                    .collect(),
            };
            ManeuverResponse {
                summary: ManeuverMetadataResponse {
                    time_seconds: value.duration as f64,
                    length_meters: value.distance.unwrap_or_default(),
                    begin_shape_index: todo!(),
                    end_shape_index: todo!(),
                    highway: None,
                    gate: None,
                    ferry: None,
                },
                travel_mode: ManeuverTravelModeResponse::from(value.mode),
                transit_info: Some(transit_info),
            }
        }
    }

    #[serde_with::skip_serializing_none]
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    struct ManeuverMetadataResponse {
        /// Estimated time along the maneuver in seconds
        #[schema(example = 201.025)]
        time_seconds: f64,
        /// Maneuver length in meters
        #[schema(example = 103.01)]
        length_meters: f64,
        /// A bounding box containing all items exactly
        bbox: BBox,
        /// Indexes where the list of shape points the maneuver starts/stops
        shape_index: Range<usize>,

        /// `true` if a highway is encountered on this maneuver
        highway: Option<bool>,
        /// `true` if a gate is encountered on this maneuver
        gate: Option<bool>,
        /// `true` if a ferry is encountered on this maneuver
        ferry: Option<bool>,
    }

    #[serde_with::skip_serializing_none]
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    struct TransitInfoResponse {
        /// Global trip identifier
        #[schema(examples("f-9q9-bart", "f-zeus~schwäbisch~gmünd~gbfs"))]
        trip_id: Option<String>,
        /// Short name describing the transit route
        #[schema(examples("N"))]
        short_name: Option<String>,
        /// Long name describing the transit route
        #[schema(examples("Broadway Express"))]
        long_name: Option<String>,
        /// The sign on a public transport vehicle that identifies the route destination to passengers
        #[schema(examples("ASTORIA - DITMARS BLVD"))]
        headsign: Option<String>,
        /// The numeric color value associated with a transit route
        ///
        /// The value for yellow would be `"#16567306"`
        #[schema(examples("#16567306"))]
        color: Option<String>,
        /// The numeric text color value associated with a transit route
        ///
        /// The value for black would be `0`
        #[schema(examples(0))]
        text_color: Option<String>,
        /// Global operator/agency identifier
        ///
        /// **Tipp:** you use these as feed-ids in transitland.
        /// Example: <https://www.transit.land/feeds/o-u281z9-mvv>
        #[schema(examples("o-u281z9-mvv"))]
        operator_id: Option<String>,
        /// Operator/agency name
        ///
        /// Short name is used over long name
        #[schema(examples(
            "BART",
            "King County Marine Division",
            "Münchner Verkehrs- und Tarifverbund (MVV)"
        ))]
        operator_name: Option<String>,
        /// Operator/agency URL
        #[schema(examples("http://web.mta.info/", "http://www.mvv-muenchen.de/"))]
        operator_url: Option<String>,
    }

    #[derive(Serialize, Debug, utoipa::ToSchema)]
    #[serde(rename_all = "snake_case")]
    enum ManeuverTravelModeResponse {
        Drive,
        Pedestrian,
        Bicycle,
        PublicTransit,
        Other,
    }
    impl From<valhalla::TravelMode> for ManeuverTravelModeResponse {
        fn from(value: valhalla::TravelMode) -> Self {
            match value {
                valhalla::TravelMode::Drive => Self::Drive,
                valhalla::TravelMode::Pedestrian => Self::Pedestrian,
                valhalla::TravelMode::Bicycle => Self::Bicycle,
                valhalla::TravelMode::Transit => Self::PublicTransit,
            }
        }
    }
    impl From<motis::Mode> for ManeuverTravelModeResponse {
        fn from(value: motis::Mode) -> Self {
            match value {
                motis::Mode::Airplane => Self::PublicTransit,
                motis::Mode::Walk => Self::Pedestrian,
                motis::Mode::Bike => Self::Bicycle,
                motis::Mode::Rental => Self::Bicycle,
                motis::Mode::Car | motis::Mode::CarParking | motis::Mode::Odm => Self::Drive,
                motis::Mode::Transit
                | motis::Mode::Tram
                | motis::Mode::Subway
                | motis::Mode::Ferry
                | motis::Mode::Metro
                | motis::Mode::Bus
                | motis::Mode::Coach
                | motis::Mode::Rail
                | motis::Mode::HighspeedRail
                | motis::Mode::LongDistance
                | motis::Mode::NightRail
                | motis::Mode::RegionalFastRail
                | motis::Mode::RegionalRail => Self::PublicTransit,
                motis::Mode::Other => Self::Other,
            }
        }
    }
}
mod step {
    use super::*;
    use crate::routes::maps::route::leg::BBox;
    use std::ops::Range;

    #[serde_with::skip_serializing_none]
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    struct StepResponse {
        /// Which icon should the router display for this step
        r#type: StepTypeResponse,
        /// Summary what happens in this step
        summary: StepMetadataResponse,
        /// Text-Instructions to either show or audibly tell the user
        instructions: InstructionStepResponse,
    }

    /// instructions associated with a step
    #[serde_with::skip_serializing_none]
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    pub struct InstructionStepResponse {
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
    }

    /// Allows differentiating an icon in the frontend
    #[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, utoipa::ToSchema)]
    #[serde(rename_all = "snake_case")]
    enum StepTypeResponse {
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
        // motis does transmit how to circle roundabouts, valhalla just enters/exits them
        CircleClockwise,
        CircleCounterclockwise,
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
    impl From<valhalla::ManeuverType> for StepTypeResponse {
        fn from(value: valhalla::ManeuverType) -> Self {
            match value {
                valhalla::ManeuverType::None => Self::None,
                valhalla::ManeuverType::Start => Self::Start,
                valhalla::ManeuverType::StartRight => Self::StartRight,
                valhalla::ManeuverType::StartLeft => Self::StartLeft,
                valhalla::ManeuverType::Destination => Self::Destination,
                valhalla::ManeuverType::DestinationRight => Self::DestinationRight,
                valhalla::ManeuverType::DestinationLeft => Self::DestinationLeft,
                valhalla::ManeuverType::Becomes => Self::Becomes,
                valhalla::ManeuverType::Continue => Self::Continue,
                valhalla::ManeuverType::SlightRight => Self::SlightRight,
                valhalla::ManeuverType::Right => Self::Right,
                valhalla::ManeuverType::SharpRight => Self::SharpRight,
                valhalla::ManeuverType::UturnRight => Self::UturnRight,
                valhalla::ManeuverType::UturnLeft => Self::UturnLeft,
                valhalla::ManeuverType::SharpLeft => Self::SharpLeft,
                valhalla::ManeuverType::Left => Self::Left,
                valhalla::ManeuverType::SlightLeft => Self::SlightLeft,
                valhalla::ManeuverType::RampStraight => Self::RampStraight,
                valhalla::ManeuverType::RampRight => Self::RampRight,
                valhalla::ManeuverType::RampLeft => Self::RampLeft,
                valhalla::ManeuverType::ExitRight => Self::ExitRight,
                valhalla::ManeuverType::ExitLeft => Self::ExitLeft,
                valhalla::ManeuverType::StayStraight => Self::StayStraight,
                valhalla::ManeuverType::StayRight => Self::StayRight,
                valhalla::ManeuverType::StayLeft => Self::StayLeft,
                valhalla::ManeuverType::Merge => Self::Merge,
                valhalla::ManeuverType::RoundaboutEnter => Self::RoundaboutEnter,
                valhalla::ManeuverType::RoundaboutExit => Self::RoundaboutExit,
                valhalla::ManeuverType::FerryEnter => Self::FerryEnter,
                valhalla::ManeuverType::FerryExit => Self::FerryExit,
                valhalla::ManeuverType::Transit => Self::Transit,
                valhalla::ManeuverType::TransitTransfer => Self::TransitTransfer,
                valhalla::ManeuverType::TransitRemainOn => Self::TransitRemainOn,
                valhalla::ManeuverType::TransitConnectionStart => Self::TransitConnectionStart,
                valhalla::ManeuverType::TransitConnectionTransfer => {
                    Self::TransitConnectionTransfer
                }
                valhalla::ManeuverType::TransitConnectionDestination => {
                    Self::TransitConnectionDestination
                }
                valhalla::ManeuverType::PostTransitConnectionDestination => {
                    Self::PostTransitConnectionDestination
                }
                valhalla::ManeuverType::MergeRight => Self::MergeRight,
                valhalla::ManeuverType::MergeLeft => Self::MergeLeft,
                valhalla::ManeuverType::ElevatorEnter => Self::ElevatorEnter,
                valhalla::ManeuverType::StepsEnter => Self::StepsEnter,
                valhalla::ManeuverType::EscalatorEnter => Self::EscalatorEnter,
                valhalla::ManeuverType::BuildingEnter => Self::BuildingEnter,
                valhalla::ManeuverType::BuildingExit => Self::BuildingExit,
            }
        }
    }
    impl From<motis::Direction> for StepTypeResponse {
        fn from(direction: motis::Direction) -> Self {
            match direction {
                motis::Direction::Depart => Self::Start,
                motis::Direction::HardLeft => Self::SharpLeft,
                motis::Direction::Left => Self::Left,
                motis::Direction::SlightlyLeft => Self::SlightLeft,
                motis::Direction::Continue => Self::Continue,
                motis::Direction::SlightlyRight => Self::SlightRight,
                motis::Direction::Right => Self::Right,
                motis::Direction::HardRight => Self::SharpRight,
                motis::Direction::CircleClockwise => Self::CircleClockwise,
                motis::Direction::CircleCounterclockwise => Self::CircleCounterclockwise,
                motis::Direction::Stairs => Self::EscalatorEnter,
                motis::Direction::Elevator => Self::ElevatorEnter,
                motis::Direction::UturnLeft => Self::UturnLeft,
                motis::Direction::UturnRight => Self::UturnRight,
            }
        }
    }

    #[serde_with::skip_serializing_none]
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    struct StepMetadataResponse {
        ///OpenStreetMap way index
        osm_way: Option<i64>,
        /// Distance traveled in meters
        #[schema(examples(60))]
        length_meters: f64,
        /// A bounding box containing all items exactly
        bbox: BBox,
        /// Indexes where the list of shape points the maneuver starts/stops
        shape_index: Range<usize>,

        /// [`level`-tag](http://wiki.openstreetmap.org/wiki/Key:level) this step starts at
        #[schema(example = 1.0)]
        from_level: f64,
        /// [`level`-tag](http://wiki.openstreetmap.org/wiki/Key:level) this step ends at
        #[schema(example = 2.0)]
        to_level: f64,
    }
    #[serde_with::skip_serializing_none]
    #[derive(Serialize, Debug, utoipa::ToSchema)]
    struct TransitStepResponse {
        /// **Not implemented!**
        /// This step is on an open area, such as a plaza or train platform,
        /// and thus the directions should say something like "cross"
        area: bool,
        /// **Not implemented!**
        /// When exiting a highway or traffic circle, the exit name/number.
        #[serde(skip_serializing_if = "String::is_empty")]
        exit: String,

        r#type: StepTypeResponse,
        /// **Not implemented!**
        /// Indicates whether a street changes direction at an intersection.
        stay_on: bool,
        ///The name of the street.
        #[schema(examples("Bolzmanstraße"))]
        #[serde(skip_serializing_if = "String::is_empty")]
        street_name: String,
    }
    impl From<motis::StepInstruction> for TransitStepResponse {
        fn from(value: motis::StepInstruction) -> Self {
            TransitStepResponse {
                area: value.area,
                exit: value.exit,
                osm_way: value.osm_way,
                length_meters: value.distance,
                r#type: StepTypeResponse::from(value.relative_direction),
                stay_on: value.stay_on,
                street_name: value.street_name,
                from_level: value.from_level,
                to_level: value.to_level,
            }
        }
    }
}
