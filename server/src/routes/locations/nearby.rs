use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
struct Transportation {
    id: String,
    name: String,
    parent_id: Option<String>,
    parent_name: Option<String>,
    /// Latitude
    #[schema(example = 48.26244490906312, nullable = false)]
    lat: Option<f64>, // not really null, sqlx just thinks this
    /// Longitude
    #[schema(example = 48.26244490906312, nullable = false)]
    lon: Option<f64>, // not really null, sqlx just thinks this
    #[schema(exclusive_minimum = 0.0, nullable = false)]
    distance_meters: Option<f64>, // not really null, sqlx just thinks this
}
#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
struct NearbyResponse {
    #[schema(max_items = 50)]
    public_transport: Vec<Transportation>,
}

#[derive(Deserialize, utoipa::IntoParams)]
struct NearbyPathParams {
    /// ID of a location
    id: String,
}

/// Get the nearby items
///
/// Shows nearby POIs like public transport stations
#[utoipa::path(
    tags=["locations"],
    params(NearbyPathParams),
    responses(
        (status = 200, description = "Things **nearby to the location**", body=NearbyResponse, content_type = "application/json"),
        (status = 404, description = "**Not found.** Make sure that requested item exists", body = String, content_type = "text/plain", example = "Not found"),
    )
)]
#[get("/api/locations/{id}/nearby")]
pub async fn nearby_handler(
    params: web::Path<NearbyPathParams>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params
        .id
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    // TODO: use the spatial index instead of just computing the distance for every entry
    let transportation = sqlx::query_as!(
        Transportation,
        r#"
WITH coodinates_for_keys(key, coordinate) as (SELECT key, point(lat, lon)::geometry as coordinate
                                              from de)

SELECT t.id,
       t.name,
       parent.id as parent_id,
       parent.name as parent_name,
       ST_X(t.coordinate::geometry)                             as lat,
       ST_Y(t.coordinate::geometry)                             as lon,
       ST_DISTANCE(t.coordinate::geometry, c.coordinate, false) as distance_meters
FROM coodinates_for_keys c,
     transportation_stations t
     LEFT OUTER JOIN transportation_stations parent on t.parent = parent.id
WHERE ST_DISTANCE(t.coordinate::geometry, c.coordinate, false) < 1000
  AND c.key = $1
ORDER BY ST_DISTANCE(t.coordinate::geometry, c.coordinate, false)
LIMIT 50"#,
        id
    )
    .fetch_all(&data.pool)
    .await;
    match transportation {
        Ok(public_transport) => HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::MaxAge(2 * 24 * 60 * 60), // valid for 2d
                CacheDirective::Public,
            ]))
            .json(NearbyResponse { public_transport }),
        Err(e) => {
            error!("Could not get nearby pois because: {e:?}");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error")
        }
    }
}
