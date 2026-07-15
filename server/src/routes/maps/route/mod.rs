use std::fmt;

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
    route::{ShapePoint, Trip},
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

// clear domination:
// pedestrian is always dominated by public transit
// if a user has car access, they likely prefer car
// we can't know if they have, so car dominates ptw
//
// => decision:
// bicycle below, then transit measured against the car it would replace
const BICYCLE_MAX_KM: f64 = 3.0;

/// However quick the drive, a default may not commit the user to longer than this
const MIN_TRANSIT_BUDGET: TimeDelta = TimeDelta::minutes(60);
/// A longer trip may take proportionally longer, so that regional rail is not strangled by the floor
const TRANSIT_BUDGET_PER_CAR_MINUTE: i32 = 3;

/// What the user commits to end to end: waiting included, from being ready until arrival
fn total_travel_time(
    itinerary: &Itinerary,
    requested_time: DateTime<Utc>,
    arrive_by: bool,
) -> TimeDelta {
    if arrive_by {
        requested_time - itinerary.start_time
    } else {
        itinerary.end_time - requested_time
    }
}

/// Whether transit is worth defaulting to, or whether this is a Garching-at-night that is "better reached by car"
fn transit_beats_the_car(
    itineraries: &[Itinerary],
    car_time: TimeDelta,
    requested_time: DateTime<Utc>,
    arrive_by: bool,
) -> bool {
    let budget = MIN_TRANSIT_BUDGET.max(car_time * TRANSIT_BUDGET_PER_CAR_MINUTE);
    itineraries
        .iter()
        .filter(|itinerary| itinerary.legs.iter().any(motis::is_transit_leg))
        .map(|itinerary| total_travel_time(itinerary, requested_time, arrive_by))
        .filter(|travel_time| *travel_time >= TimeDelta::zero())
        .min()
        .is_some_and(|best| best <= budget)
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

/// Which router answered, carrying what it answered with
#[derive(Debug)]
enum Router {
    Motis(Box<PlanResponse>),
    Valhalla(Box<Trip>),
}
impl Router {
    async fn resolve(
        args: &RoutingRequest,
        data: &crate::AppData,
        from: Coordinate,
        to: Coordinate,
    ) -> anyhow::Result<Self> {
        match args.route_costing {
            Some(CostingRequest::PublicTransit) => Ok(Self::Motis(Box::new(
                Self::via_motis(args, data, from, to).await?,
            ))),
            Some(chosen) => Ok(Self::Valhalla(Box::new(
                Self::via_valhalla(args, data, from, to, chosen).await?,
            ))),
            None => Self::smart_default(args, data, from, to).await,
        }
    }

    /// Cycling wins the short trips outright; beyond that only the clock can say whether transit is worth it,
    /// so we ask both routers and let the itinerary answer against the car it would replace.
    async fn smart_default(
        args: &RoutingRequest,
        data: &crate::AppData,
        from: Coordinate,
        to: Coordinate,
    ) -> anyhow::Result<Self> {
        if from.distance_to(&to) < BICYCLE_MAX_KM {
            return Ok(Self::Valhalla(Box::new(
                Self::via_valhalla(args, data, from, to, CostingRequest::Bicycle).await?,
            )));
        }
        let (transit, car) = tokio::join!(
            Self::via_motis(args, data, from, to),
            Self::via_valhalla(args, data, from, to, CostingRequest::Car),
        );
        let requested_time = args.time.unwrap_or_else(Utc::now);
        match (transit, car) {
            (Ok(transit), Ok(car)) => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "valhalla reports whole seconds as f64; a route long enough to overflow i64 seconds does not exist"
                )]
                let car_time = TimeDelta::seconds(car.summary.time as i64);
                if transit_beats_the_car(
                    &transit.itineraries,
                    car_time,
                    requested_time,
                    args.arrive_by,
                ) {
                    Ok(Self::Motis(Box::new(transit)))
                } else {
                    debug!(%requested_time, "transit is not worth it against the car, defaulting to the car");
                    Ok(Self::Valhalla(Box::new(car)))
                }
            }
            (Ok(transit), Err(e)) => {
                warn!(error=?e, "no car to measure transit against, keeping transit");
                Ok(Self::Motis(Box::new(transit)))
            }
            (Err(e), Ok(car)) => {
                warn!(error=?e, "could not reach transit, defaulting to the car");
                Ok(Self::Valhalla(Box::new(car)))
            }
            (Err(e), Err(_)) => Err(e),
        }
    }

    async fn via_motis(
        args: &RoutingRequest,
        data: &crate::AppData,
        from: Coordinate,
        to: Coordinate,
    ) -> anyhow::Result<PlanResponse> {
        data.motis
            .route(
                &from.to_string(),
                &to.to_string(),
                args.page_cursor.as_deref(),
                args.time.as_ref(),
                args.arrive_by,
                args.lang == LanguageOptions::En,
                args.pedestrian_type.into(),
            )
            .await
    }

    async fn via_valhalla(
        args: &RoutingRequest,
        data: &crate::AppData,
        from: Coordinate,
        to: Coordinate,
        costing: CostingRequest,
    ) -> anyhow::Result<Trip> {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "valhalla's API takes f32 coordinates; f64→f32 is acceptable, ~7 significant digits is well below the precision routing requires"
        )]
        data.valhalla
            .route(
                (from.lat as f32, from.lon as f32),
                (to.lat as f32, to.lon as f32),
                costing.to_valhalla(args.pedestrian_type, args.bicycle_type, args.ptw_type),
                args.lang == LanguageOptions::En,
            )
            .await
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
    let router = match Router::resolve(&args, &data, from, to).await {
        Ok(router) => router,
        Err(e) => {
            error!(error=?e,"error routing");
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Could not generate a route, please try again later");
        }
    };
    debug!(routing_solution=?router,"got routing solution");

    match router {
        Router::Motis(response) => HttpResponse::Ok().json(RoutingResponse::Motis(
            motis::MotisRoutingResponse::from(*response),
        )),
        Router::Valhalla(response) => HttpResponse::Ok().json(RoutingResponse::Valhalla(
            valhalla::ValhallaRoutingResponse::from(*response),
        )),
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
    #[case::same_place(MERIDIAN_ORIGIN, true)]
    #[case::just_below_the_bicycle_threshold(north_of_origin(2.9), true)]
    #[case::just_above_the_bicycle_threshold(north_of_origin(3.1), false)]
    #[case::garching(GARCHING_MI, false)]
    fn cycling_wins_only_the_short_trips(#[case] to: Coordinate, #[case] expected: bool) {
        let from = if to == GARCHING_MI {
            STAMMGELAENDE
        } else {
            MERIDIAN_ORIGIN
        };

        assert_eq!(from.distance_to(&to) < BICYCLE_MAX_KM, expected);
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

    fn requested() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 15, 3, 0, 0).unwrap()
    }

    /// One itinerary arriving `total_minutes` after the requested departure
    fn arriving_after(total_minutes: i64, modes: &[Mode]) -> Vec<Itinerary> {
        let end = requested() + TimeDelta::minutes(total_minutes);
        vec![itinerary(end - TimeDelta::minutes(20), end, modes)]
    }

    /// Measured against api.transitous.org and OSRM on 2026-07-16, the trips the exposé names.
    /// Wait is included, so these are door-to-door from the requested departure.
    #[rstest]
    #[case::garching_at_0000_ubahn_still_runs(49, 20, true)]
    #[case::garching_at_0100_a_four_hour_ordeal(245, 20, false)]
    #[case::garching_at_0200(185, 20, false)]
    #[case::garching_at_0300_short_wait_but_92min_ride(125, 20, false)]
    #[case::main_campus_at_night_has_a_night_service(45, 10, true)]
    #[case::weihenstephan_by_day_regional_rail(75, 36, true)]
    #[case::weihenstephan_in_the_evening(82, 36, true)]
    #[case::weihenstephan_at_0100(202, 36, false)]
    #[case::weihenstephan_at_0300(142, 36, false)]
    #[case::munich_to_berlin_by_ice_beats_driving(264, 353, true)]
    fn the_expose_examples(
        #[case] transit_minutes: i64,
        #[case] car_minutes: i64,
        #[case] transit_is_the_default: bool,
    ) {
        let itineraries = arriving_after(transit_minutes, &[Mode::Walk, Mode::Bus, Mode::Walk]);

        assert_eq!(
            transit_beats_the_car(
                &itineraries,
                TimeDelta::minutes(car_minutes),
                requested(),
                false
            ),
            transit_is_the_default
        );
    }

    #[rstest]
    #[case::a_bus_counts(&[Mode::Walk, Mode::Bus, Mode::Walk], true)]
    #[case::on_demand_transport_counts(&[Mode::Walk, Mode::Odm], true)]
    #[case::walking_the_whole_way_is_not_transit(&[Mode::Walk], false)]
    #[case::a_rental_bike_is_not_transit(&[Mode::Walk, Mode::Rental, Mode::Walk], false)]
    fn only_itineraries_that_actually_use_transit_count(
        #[case] modes: &[Mode],
        #[case] expected: bool,
    ) {
        let comfortably_within_budget = arriving_after(30, modes);

        assert_eq!(
            transit_beats_the_car(
                &comfortably_within_budget,
                TimeDelta::minutes(20),
                requested(),
                false
            ),
            expected
        );
    }

    #[test]
    fn a_quick_drive_does_not_drag_the_budget_below_the_floor() {
        // 3x a 5min drive is 15min, but a 40min trip is still a fine default
        let itineraries = arriving_after(40, &[Mode::Walk, Mode::Tram]);

        assert!(transit_beats_the_car(
            &itineraries,
            TimeDelta::minutes(5),
            requested(),
            false
        ));
    }

    #[test]
    fn the_best_itinerary_decides_not_the_first() {
        let mut itineraries = arriving_after(240, &[Mode::Walk, Mode::Bus]);
        itineraries.extend(arriving_after(40, &[Mode::Walk, Mode::Subway]));

        assert!(transit_beats_the_car(
            &itineraries,
            TimeDelta::minutes(20),
            requested(),
            false
        ));
    }

    #[test]
    fn arriving_by_is_measured_back_from_the_requested_arrival() {
        let leaves_35min_early = vec![itinerary(
            requested() - TimeDelta::minutes(35),
            requested() - TimeDelta::minutes(5),
            &[Mode::Bus],
        )];

        assert!(transit_beats_the_car(
            &leaves_35min_early,
            TimeDelta::minutes(20),
            requested(),
            true
        ));
        // the same itinerary is useless to someone departing at the requested time
        assert!(!transit_beats_the_car(
            &leaves_35min_early,
            TimeDelta::minutes(20),
            requested(),
            false
        ));
    }
}
