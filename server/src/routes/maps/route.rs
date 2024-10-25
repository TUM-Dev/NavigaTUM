use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, error, info};
use valhalla_client::{Location, Manifest, Valhalla};

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, utoipa::ToSchema)]
struct Coordinate {
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, utoipa::ToSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
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

#[derive(Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
struct RoutingRequest {
    /// Start of the route
    from: RequestedLocation,
    /// Destination of the route
    to: RequestedLocation,
    /// Transport mode the user wants to use
    route_costing: CostingRequest,
}

/// Handles routing requests using provided origin (`from`) and destination (`to`) locations.
///
/// The user specifies a transport mode (`route_costing`) to tune their routing between the two locations.
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
    let to = args.from.try_resolve_coordinates(&data.pool).await;
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
    debug!(?from, ?to, "routing request");
    let base_url = "https://nav.tum.de/valhalla".parse().unwrap();
    let valhalla = Valhalla::new(base_url);
    let manifest = Manifest {
        locations: vec![
            Location::new(from.lat, from.lon),
            Location::new(to.lat, to.lon),
        ],
        costing: valhalla_client::Costing::Auto,
        ..Default::default()
    };

    let response = valhalla.route(manifest).unwrap();

    info!("got routing solution {:#?}", response);

    HttpResponse::Ok().json(RoutingResponse {})
}
#[derive(Serialize, Debug, utoipa::ToSchema)]
struct RoutingResponse {}
