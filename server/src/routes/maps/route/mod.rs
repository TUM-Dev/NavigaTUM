use std::fmt;

use crate::external::motis::MotisWrapper;
use crate::localisation::LanguageOptions;
use actix_web::{HttpResponse, get, web};
use chrono::{DateTime, TimeDelta, Utc};
use motis_openapi_progenitor::types::{Itinerary, PedestrianProfile, PlanResponse};
use serde::{Deserialize, Serialize, de};
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use serde_json::json;
use sqlx::PgPool;
use tracing::{debug, error, warn};
use valhalla_client::{
    costing::{
        AutoCostingOptions, BicycleCostingOptions, Costing, MotorScooterCostingOptions,
        MotorcycleCostingOptions, MultimodalCostingOptions, PedestrianCostingOptions,
        TransitCostingOptions, bicycle::BicycleType, pedestrian::PedestrianType,
    },
    route::ShapePoint,
};
pub(crate) mod motis;
mod valhalla;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, utoipa::ToSchema)]
struct Coordinate {
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
}
const EARTH_RADIUS_KM: f64 = 6371.0;

impl Coordinate {
    /// Great-circle distance (Haversine formula)
    fn distance_to(&self, other: &Self) -> f64 {
        let (lat1, lon1) = (self.lat.to_radians(), self.lon.to_radians());
        let (lat2, lon2) = (other.lat.to_radians(), other.lon.to_radians());

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }
}
impl From<ShapePoint> for Coordinate {
    fn from(value: ShapePoint) -> Self {
        Self {
            lon: value.lon,
            lat: value.lat,
        }
    }
}
impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.lat, self.lon)
    }
}
impl<'de> Deserialize<'de> for Coordinate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (p1, p2) = s
            .split_once(',')
            .ok_or(de::Error::custom("expected 'lat,lon'"))?;
        let lat = p1
            .parse::<f64>()
            .map_err(|e| de::Error::custom(format!("invalid latitude: {e}")))?;
        let lon = p2
            .parse::<f64>()
            .map_err(|e| de::Error::custom(format!("invalid longitude: {e}")))?;
        Ok(Self { lat, lon })
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
            Self::Coordinate(coords) => Ok(Some(*coords)),
            Self::Location(key) => {
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

const BICYCLE_MAX_KM: f64 = 3.0;
const PUBLIC_TRANSIT_MAX_KM: f64 = 40.0;

/// What the beeline distance alone suggests, before asking whether transit actually runs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DistanceBand {
    Bicycle,
    PublicTransit,
    Car,
}
impl DistanceBand {
    fn between(from: Coordinate, to: Coordinate) -> Self {
        // clear domination:
        // pedestrian is always dominated by public transit
        // if a user has car access, they likely prefer car
        // we can't know if they have, so car dominates ptw
        //
        // unclear domination (these have holes):
        // bicycle for small distances is good, but after 3km, public transit is better
        // car is better after 40km
        //
        // => decision:
        // car vs public transit vs bicycle
        let distance_km = from.distance_to(&to);
        if distance_km < BICYCLE_MAX_KM {
            Self::Bicycle
        } else if distance_km < PUBLIC_TRANSIT_MAX_KM {
            Self::PublicTransit
        } else {
            Self::Car
        }
    }
}
impl From<DistanceBand> for CostingRequest {
    fn from(value: DistanceBand) -> Self {
        match value {
            DistanceBand::Bicycle => Self::Bicycle,
            DistanceBand::PublicTransit => Self::PublicTransit,
            DistanceBand::Car => Self::Car,
        }
    }
}

/// How long a user may wait for transit before driving becomes the friendlier default
const MAX_TRANSIT_SLACK: TimeDelta = TimeDelta::hours(1);

/// Whether transit actually serves the requested time, as opposed to MOTIS answering with a walk-only itinerary or the first bus hours later
fn serves_requested_time(
    itineraries: &[Itinerary],
    requested_time: DateTime<Utc>,
    arrive_by: bool,
) -> bool {
    itineraries.iter().any(|itinerary| {
        let relevant_time = if arrive_by {
            itinerary.end_time
        } else {
            itinerary.start_time
        };
        itinerary.legs.iter().any(motis::is_transit_leg)
            && (relevant_time - requested_time).abs() <= MAX_TRANSIT_SLACK
    })
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
impl CostingRequest {
    fn to_valhalla(
        self,
        pedestrian_type: PedestrianTypeRequest,
        bicycle_type: BicycleRestrictionRequest,
        ptw_type: PoweredTwoWheeledRestrictionRequest,
    ) -> Costing {
        match self {
            Self::Pedestrian => Costing::Pedestrian(
                PedestrianCostingOptions::builder().r#type(PedestrianType::from(pedestrian_type)),
            ),
            Self::Bicycle => Costing::Bicycle(
                BicycleCostingOptions::builder().bicycle_type(BicycleType::from(bicycle_type)),
            ),
            Self::Motorcycle => match ptw_type {
                PoweredTwoWheeledRestrictionRequest::Moped => {
                    Costing::Motorcycle(MotorcycleCostingOptions::default())
                }
                PoweredTwoWheeledRestrictionRequest::Motorcycle => {
                    Costing::MotorScooter(MotorScooterCostingOptions::default())
                }
            },
            Self::Car => Costing::Auto(AutoCostingOptions::default()),
            Self::PublicTransit => {
                let pedestrian_costing = PedestrianCostingOptions::builder()
                    .r#type(PedestrianType::from(pedestrian_type));
                Costing::Multimodal(
                    MultimodalCostingOptions::builder()
                        .pedestrian(pedestrian_costing)
                        .transit(TransitCostingOptions::default()),
                )
            }
        }
    }
}

/// Where a transit routing came from, which decides whether we may still revise it
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitChoice {
    /// The user asked for transit, so we serve transit or nothing
    Explicit,
    /// We picked transit ourselves, so we may fall back if it does not pan out
    OurDefault,
}

enum Router {
    /// Carries the plan the probe already fetched, so that we don't ask MOTIS the same question twice
    Motis(Box<PlanResponse>),
    Valhalla(CostingRequest),
}
impl Router {
    async fn resolve(
        args: &RoutingRequest,
        motis: &MotisWrapper,
        from: Coordinate,
        to: Coordinate,
    ) -> anyhow::Result<Self> {
        let choice = match args.route_costing {
            Some(CostingRequest::PublicTransit) => TransitChoice::Explicit,
            Some(explicit) => return Ok(Self::Valhalla(explicit)),
            None => match DistanceBand::between(from, to) {
                DistanceBand::PublicTransit => TransitChoice::OurDefault,
                band => return Ok(Self::Valhalla(band.into())),
            },
        };
        let plan = motis
            .route(
                &from.to_string(),
                &to.to_string(),
                args.page_cursor.as_deref(),
                args.time.as_ref(),
                args.arrive_by,
                args.lang == LanguageOptions::En,
                args.pedestrian_type.into(),
            )
            .await;
        Self::from_transit_plan(
            choice,
            args.time.unwrap_or_else(Utc::now),
            args.arrive_by,
            plan,
        )
    }

    fn from_transit_plan(
        choice: TransitChoice,
        requested_time: DateTime<Utc>,
        arrive_by: bool,
        plan: anyhow::Result<PlanResponse>,
    ) -> anyhow::Result<Self> {
        if choice == TransitChoice::Explicit {
            return Ok(Self::Motis(Box::new(plan?)));
        }
        match plan {
            Ok(plan) if serves_requested_time(&plan.itineraries, requested_time, arrive_by) => {
                Ok(Self::Motis(Box::new(plan)))
            }
            Ok(_) => {
                debug!(%requested_time, "no transit serves the requested time, defaulting to the car");
                Ok(Self::Valhalla(CostingRequest::Car))
            }
            Err(e) => {
                warn!(error=?e, "could not probe transit, defaulting to the car");
                Ok(Self::Valhalla(CostingRequest::Car))
            }
        }
    }
}

#[derive(Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
struct RoutingRequest {
    #[serde(default)]
    #[param(inline)]
    lang: LanguageOptions,
    /// Start of the route
    #[param(inline)]
    from: RequestedLocation,
    /// Destination of the route
    #[param(inline)]
    to: RequestedLocation,
    /// Transport mode the user wants to use
    ///
    /// If not specified, the default is based on how far the destinations are apart and requested time.
    #[param(inline)]
    route_costing: Option<CostingRequest>,
    /// Does the user have specific walking restrictions?
    #[serde(default)]
    #[param(inline)]
    pedestrian_type: PedestrianTypeRequest,
    /// Does the user prefer mopeds or motorcycles for powered two-wheeled (ptw)?
    #[serde(default)]
    #[param(inline)]
    ptw_type: PoweredTwoWheeledRestrictionRequest,
    /// Which kind of bicycle do you ride?
    #[serde(default)]
    #[param(inline)]
    bicycle_type: BicycleRestrictionRequest,
    /// Cursor position for pagination
    /// Only avaliable for some costings
    #[serde(default)]
    #[param(inline)]
    page_cursor: Option<String>,
    /// Time for the route (ISO 8601 format)
    /// Used with `arrive_by` to determine if this is departure or arrival time
    #[serde(default)]
    #[param(inline)]
    time: Option<DateTime<Utc>>,
    /// Whether the time parameter represents arrival time (true) or departure time (false/not set)
    #[serde(default)]
    #[param(inline)]
    arrive_by: bool,
}

/// Does the user have specific walking needs?
#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum PedestrianTypeRequest {
    #[default]
    Standard,
    Blind,
    Wheelchair,
}

impl From<PedestrianTypeRequest> for PedestrianType {
    fn from(value: PedestrianTypeRequest) -> Self {
        match value {
            PedestrianTypeRequest::Standard => Self::Foot,
            PedestrianTypeRequest::Blind => Self::Blind,
            PedestrianTypeRequest::Wheelchair => Self::Wheelchair,
        }
    }
}
impl From<PedestrianTypeRequest> for PedestrianProfile {
    fn from(value: PedestrianTypeRequest) -> Self {
        match value {
            PedestrianTypeRequest::Standard | PedestrianTypeRequest::Blind => Self::Foot,
            PedestrianTypeRequest::Wheelchair => Self::Wheelchair,
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
            BicycleRestrictionRequest::Road => Self::Road,
            BicycleRestrictionRequest::Hybrid => Self::Hybrid,
            BicycleRestrictionRequest::Cross => Self::Cross,
            BicycleRestrictionRequest::Mountain => Self::Mountain,
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
#[get("/api/maps/route", wrap = "actix_middleware_etag::Etag::default()")]
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
    let router = match Router::resolve(&args, &data.motis, from, to).await {
        Ok(router) => router,
        Err(e) => {
            error!(error=?e,"error routing");
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Could not generate a route, please try again later");
        }
    };

    match router {
        Router::Motis(response) => {
            debug!(routing_solution=?response,"got routing solution");

            HttpResponse::Ok().json(RoutingResponse::Motis(motis::MotisRoutingResponse::from(
                *response,
            )))
        }
        Router::Valhalla(costing) => {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "valhalla's API takes f32 coordinates; f64→f32 is acceptable, ~7 significant digits is well below the precision routing requires"
            )]
            let routing = data
                .valhalla
                .route(
                    (from.lat as f32, from.lon as f32),
                    (to.lat as f32, to.lon as f32),
                    costing.to_valhalla(args.pedestrian_type, args.bicycle_type, args.ptw_type),
                    args.lang == LanguageOptions::En,
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

            HttpResponse::Ok().json(RoutingResponse::Valhalla(
                valhalla::ValhallaRoutingResponse::from(response),
            ))
        }
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(tag = "router")]
#[serde(rename_all = "snake_case")]
enum RoutingResponse {
    Valhalla(valhalla::ValhallaRoutingResponse),
    Motis(motis::MotisRoutingResponse),
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        reason = "tests assert via panic/unwrap"
    )]
    use std::collections::HashMap;
    use std::f64::consts::PI;

    use chrono::TimeZone as _;
    use motis_openapi_progenitor::types::{EncodedPolyline, Leg, Mode, Place};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    const STAMMGELAENDE: Coordinate = Coordinate {
        lat: 48.149_66,
        lon: 11.567_94,
    };
    const GARCHING_MI: Coordinate = Coordinate {
        lat: 48.262_44,
        lon: 11.667_99,
    };
    const MERIDIAN_ORIGIN: Coordinate = Coordinate {
        lat: 48.0,
        lon: 11.0,
    };

    const KM_PER_DEGREE_LAT: f64 = EARTH_RADIUS_KM * PI / 180.0;

    fn north_of_origin(distance_km: f64) -> Coordinate {
        Coordinate {
            lat: MERIDIAN_ORIGIN.lat + distance_km / KM_PER_DEGREE_LAT,
            lon: MERIDIAN_ORIGIN.lon,
        }
    }

    #[test]
    fn distance_is_measured_in_km() {
        let distance = STAMMGELAENDE.distance_to(&GARCHING_MI);
        assert!(
            (14.0..15.0).contains(&distance),
            "the ~14.5km from the Stammgelände to Garching came out as {distance}"
        );
    }

    #[rstest]
    #[case::same_place(MERIDIAN_ORIGIN, DistanceBand::Bicycle)]
    #[case::just_below_the_bicycle_threshold(north_of_origin(2.9), DistanceBand::Bicycle)]
    #[case::just_above_the_bicycle_threshold(north_of_origin(3.1), DistanceBand::PublicTransit)]
    #[case::just_below_the_transit_threshold(north_of_origin(39.9), DistanceBand::PublicTransit)]
    #[case::just_above_the_transit_threshold(north_of_origin(40.1), DistanceBand::Car)]
    #[case::munich_to_berlin(Coordinate { lat: 52.520_01, lon: 13.404_95 }, DistanceBand::Car)]
    fn distance_bands(#[case] to: Coordinate, #[case] expected: DistanceBand) {
        assert_eq!(DistanceBand::between(MERIDIAN_ORIGIN, to), expected);
    }

    fn place() -> Place {
        Place::builder()
            .lat(GARCHING_MI.lat)
            .level(0.0)
            .lon(GARCHING_MI.lon)
            .name("somewhere")
            .try_into()
            .unwrap()
    }

    fn leg(mode: Mode, start: DateTime<Utc>, end: DateTime<Utc>) -> Leg {
        let geometry: EncodedPolyline = EncodedPolyline::builder()
            .length(0)
            .points(String::new())
            .precision(7)
            .try_into()
            .unwrap();
        Leg::builder()
            .duration((end - start).num_seconds())
            .start_time(start)
            .end_time(end)
            .scheduled_start_time(start)
            .scheduled_end_time(end)
            .from(place())
            .to(place())
            .leg_geometry(geometry)
            .mode(mode)
            .real_time(false)
            .scheduled(true)
            .try_into()
            .unwrap()
    }

    fn itinerary(start: DateTime<Utc>, end: DateTime<Utc>, modes: &[Mode]) -> Itinerary {
        Itinerary::builder()
            .duration((end - start).num_seconds())
            .start_time(start)
            .end_time(end)
            .legs(
                modes
                    .iter()
                    .map(|m| leg(*m, start, end))
                    .collect::<Vec<_>>(),
            )
            .transfers(0)
            .try_into()
            .unwrap()
    }

    fn plan(itineraries: Vec<Itinerary>) -> PlanResponse {
        PlanResponse::builder()
            .debug_output(HashMap::new())
            .direct(Vec::new())
            .from(place())
            .to(place())
            .itineraries(itineraries)
            .next_page_cursor(String::new())
            .previous_page_cursor(String::new())
            .request_parameters(HashMap::new())
            .try_into()
            .unwrap()
    }

    fn night() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 15, 3, 0, 0).unwrap()
    }

    #[rstest]
    #[case::a_bus_at_the_requested_time(&[Mode::Walk, Mode::Bus, Mode::Walk], 5, true)]
    #[case::a_subway_a_little_later(&[Mode::Walk, Mode::Subway], 55, true)]
    #[case::on_demand_transport_counts(&[Mode::Walk, Mode::Odm], 5, true)]
    #[case::walking_the_whole_way_is_not_transit(&[Mode::Walk], 5, false)]
    #[case::a_rental_bike_is_not_transit(&[Mode::Walk, Mode::Rental, Mode::Walk], 5, false)]
    #[case::the_first_bus_hours_later(&[Mode::Walk, Mode::Bus], 150, false)]
    fn transit_serving_the_requested_time(
        #[case] modes: &[Mode],
        #[case] departs_in_minutes: i64,
        #[case] expected: bool,
    ) {
        let start = night() + TimeDelta::minutes(departs_in_minutes);
        let itineraries = vec![itinerary(start, start + TimeDelta::minutes(30), modes)];

        assert_eq!(
            serves_requested_time(&itineraries, night(), false),
            expected
        );
    }

    #[test]
    fn arriving_by_is_judged_on_the_arrival_time() {
        let requested = night();
        let departs_long_before_but_arrives_on_time = vec![itinerary(
            requested - TimeDelta::hours(3),
            requested - TimeDelta::minutes(5),
            &[Mode::Bus],
        )];

        assert!(serves_requested_time(
            &departs_long_before_but_arrives_on_time,
            requested,
            true
        ));
        assert!(!serves_requested_time(
            &departs_long_before_but_arrives_on_time,
            requested,
            false
        ));
    }

    #[test]
    fn a_single_usable_itinerary_among_many_is_enough() {
        let itineraries = vec![
            itinerary(night(), night() + TimeDelta::hours(2), &[Mode::Walk]),
            itinerary(
                night() + TimeDelta::hours(4),
                night() + TimeDelta::hours(5),
                &[Mode::Subway],
            ),
            itinerary(
                night() + TimeDelta::minutes(10),
                night() + TimeDelta::minutes(40),
                &[Mode::Bus],
            ),
        ];

        assert!(serves_requested_time(&itineraries, night(), false));
    }

    fn walk_only_plan() -> PlanResponse {
        plan(vec![itinerary(
            night(),
            night() + TimeDelta::hours(3),
            &[Mode::Walk],
        )])
    }

    fn plan_with_a_bus() -> PlanResponse {
        plan(vec![itinerary(
            night() + TimeDelta::minutes(5),
            night() + TimeDelta::minutes(35),
            &[Mode::Walk, Mode::Bus],
        )])
    }

    #[test]
    fn a_night_time_route_to_garching_falls_back_to_the_car() {
        assert_eq!(
            DistanceBand::between(STAMMGELAENDE, GARCHING_MI),
            DistanceBand::PublicTransit
        );

        let router = Router::from_transit_plan(
            TransitChoice::OurDefault,
            night(),
            false,
            Ok(walk_only_plan()),
        )
        .unwrap();

        assert!(matches!(router, Router::Valhalla(CostingRequest::Car)));
    }

    #[test]
    fn transit_serving_the_requested_time_is_kept() {
        let router = Router::from_transit_plan(
            TransitChoice::OurDefault,
            night(),
            false,
            Ok(plan_with_a_bus()),
        )
        .unwrap();

        assert!(matches!(router, Router::Motis(_)));
    }

    #[test]
    fn an_explicit_transit_choice_is_never_revised() {
        let router = Router::from_transit_plan(
            TransitChoice::Explicit,
            night(),
            false,
            Ok(walk_only_plan()),
        )
        .unwrap();

        assert!(matches!(router, Router::Motis(_)));
    }

    #[test]
    fn an_unreachable_motis_falls_back_to_the_car() {
        let router = Router::from_transit_plan(
            TransitChoice::OurDefault,
            night(),
            false,
            Err(anyhow::anyhow!("motis is down")),
        )
        .unwrap();

        assert!(matches!(router, Router::Valhalla(CostingRequest::Car)));
    }

    #[test]
    fn an_unreachable_motis_surfaces_when_transit_was_explicitly_asked_for() {
        let router = Router::from_transit_plan(
            TransitChoice::Explicit,
            night(),
            false,
            Err(anyhow::anyhow!("motis is down")),
        );

        assert!(router.is_err());
    }
}
