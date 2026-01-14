use crate::localisation::LanguageOptions;
use actix_web::{HttpResponse, get, web};
use serde::{Deserialize, Serialize};
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use serde_json::json;
use sqlx::PgPool;
use tracing::{debug, error};
use valhalla_client::{
    costing::{
        BicycleCostingOptions, Costing, MultimodalCostingOptions, PedestrianCostingOptions,
        bicycle::BicycleType, pedestrian::PedestrianType,
    },
    route::ShapePoint,
};
mod motis;
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
impl Coordinate {
    /// Great-circle distance (Haversine formula)
    fn distance_to(&self, other: &Coordinate) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

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
        Coordinate {
            lon: value.lon,
            lat: value.lat,
        }
    }
}
impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.lat, self.lon)
    }
}
impl<'de> serde::Deserialize<'de> for Coordinate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (p1, p2) = s
            .split_once(',')
            .ok_or(serde::de::Error::custom("expected 'lat,lon'"))?;
        let lat = p1
            .parse::<f64>()
            .map_err(|_| serde::de::Error::custom("invalid latitude"))?;
        let lon = p2
            .parse::<f64>()
            .map_err(|_| serde::de::Error::custom("invalid longitude"))?;
        Ok(Coordinate { lat, lon })
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
impl CostingRequest {
    fn smart_default(route_costing: Option<Self>, from: Coordinate, to: Coordinate) -> Self {
        if let Some(cost) = route_costing {
            cost
        } else {
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
            if from.distance_to(&to) < 3000.0 {
                Self::Bicycle
            } else if from.distance_to(&to) < 40000.0 {
                Self::PublicTransit
            } else {
                Self::Car
            }
        }
    }
    fn to_valhalla(
        self,
        pedestrian_type: PedestrianTypeRequest,
        bicycle_type: BicycleRestrictionRequest,
        ptw_type: PoweredTwoWheeledRestrictionRequest,
    ) -> Costing {
        match self {
            CostingRequest::Pedestrian => Costing::Pedestrian(
                PedestrianCostingOptions::builder().r#type(PedestrianType::from(pedestrian_type)),
            ),
            CostingRequest::Bicycle => Costing::Bicycle(
                BicycleCostingOptions::builder().bicycle_type(BicycleType::from(bicycle_type)),
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
                    .r#type(PedestrianType::from(pedestrian_type));
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
    /// Used with arrive_by to determine if this is departure or arrival time
    #[serde(default)]
    #[param(inline)]
    time: Option<chrono::DateTime<chrono::Utc>>,
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
            PedestrianTypeRequest::Standard => PedestrianType::Foot,
            PedestrianTypeRequest::Blind => PedestrianType::Blind,
            PedestrianTypeRequest::Wheelchair => PedestrianType::Wheelchair,
        }
    }
}
impl From<PedestrianTypeRequest> for motis_openapi_progenitor::types::PedestrianProfile {
    fn from(value: PedestrianTypeRequest) -> Self {
        match value {
            PedestrianTypeRequest::Standard | PedestrianTypeRequest::Blind => {
                motis_openapi_progenitor::types::PedestrianProfile::Foot
            }
            PedestrianTypeRequest::Wheelchair => {
                motis_openapi_progenitor::types::PedestrianProfile::Wheelchair
            }
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
    let costing = CostingRequest::smart_default(args.route_costing, from, to);

    if costing == CostingRequest::PublicTransit {
        let routing = data
            .motis
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

        HttpResponse::Ok().json(RoutingResponse::Motis(motis::MotisRoutingResponse::from(
            response,
        )))
    } else {
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

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(tag = "router")]
#[serde(rename_all = "snake_case")]
enum RoutingResponse {
    Valhalla(valhalla::ValhallaRoutingResponse),
    Motis(motis::MotisRoutingResponse),
}
