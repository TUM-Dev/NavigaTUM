use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{debug, error, info};
use valhalla_client::{Location, Manifest, Valhalla};

#[derive(Deserialize, Clone, Copy, Debug)]
struct Coordinate {
    lat: f64,
    lon: f64,
}

#[derive(Deserialize, Clone, Debug)]
enum RequestedLocation {
    Coordinate(Coordinate),
    Location(String),
}
impl RequestedLocation {
    async fn try_resolve_coordinates(
        &self,
        pool: &PgPool,
    ) -> actix_web::Result<Coordinate, HttpResponse> {
        match self {
            RequestedLocation::Coordinate(coords) => Ok(*coords),
            RequestedLocation::Location(key) => {
                let coords = sqlx::query_as!(Coordinate, "SELECT lat,lon FROM de WHERE key = $1 and lat IS NOT NULL and lon IS NOT NULL", key)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| {
                        error!("could not resolve {key} into coordinates because {e:?}");
                        HttpResponse::InternalServerError()
                            .content_type("text/plain")
                            .body("Failed to resolve key")
                    })?;
                if let Some(coords) = coords {
                    Ok(coords)
                } else {
                    Err(HttpResponse::NotFound()
                        .content_type("text/plain")
                        .body("Not found"))
                }
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct RoutingRequest {
    from: RequestedLocation,
    to: RequestedLocation,
}

#[get("/route")]
pub async fn route_handler(
    args: web::Query<RoutingRequest>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let from = match args.from.try_resolve_coordinates(&data.pool).await {
        Ok(c) => c,
        Err(e) => {
            return e;
        }
    };
    let to = match args.to.try_resolve_coordinates(&data.pool).await {
        Ok(c) => c,
        Err(e) => {
            return e;
        }
    };
    debug!("routing request from {from:?} to {to:?}");
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

    HttpResponse::Ok().body("successfull routing solution")
}
