use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Transportation {
    id: String,
    name: String,
    parent_id: Option<String>,
    parent_name: Option<String>,
    /// not really null, sqlx just thinks this
    lat: Option<f64>,
    /// not really null, sqlx just thinks this
    lon: Option<f64>,
    /// not really null, sqlx just thinks this
    distance_meters: Option<f64>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct NearbyResponse {
    public_transport: Vec<Transportation>,
}

#[get("/api/location/{id}/nearby")]
pub async fn nearby_handler(
    params: web::Path<String>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params.into_inner();
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
        Ok(public_transport) => HttpResponse::Ok().json(NearbyResponse { public_transport }),
        Err(e) => {
            error!("Could not get nearby pois because: {e:?}");
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error");
        }
    }
}
