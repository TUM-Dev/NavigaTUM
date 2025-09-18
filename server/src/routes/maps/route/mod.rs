use crate::localisation::LanguageOptions;
use actix_web::{HttpResponse, get, web};
use serde::{Deserialize, Serialize};
#[expect(
    unused_imports,
    reason = "has to be imported as otherwise utoipa generates incorrect code"
)]
use serde_json::json;
use sqlx::PgPool;
use std::ops::Deref;
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
    #[param(inline)]
    lang: LanguageOptions,
    /// Start of the route
    #[param(inline)]
    from: RequestedLocation,
    /// Destination of the route
    #[param(inline)]
    to: RequestedLocation,
    /// Transport mode the user wants to use
    #[param(inline)]
    route_costing: CostingRequest,
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
        let routing = data.motis.route("", "").await;
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

        HttpResponse::Ok().json(RoutingResponse::Motis(motis::RoutingResponse::from(
            response,
        )))
    } else {
        let routing = data
            .valhalla
            .route(
                (from.lat as f32, from.lon as f32),
                (to.lat as f32, to.lon as f32),
                Costing::from(args.deref()),
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

        HttpResponse::Ok().json(RoutingResponse::Valhalla(valhalla::RoutingResponse::from(
            response,
        )))
    }
}

#[derive(Serialize, Debug, utoipa::ToSchema)]
#[serde(tag = "type")]
enum RoutingResponse {
    Valhalla(valhalla::RoutingResponse),
    Motis(motis::RoutingResponse),
}
