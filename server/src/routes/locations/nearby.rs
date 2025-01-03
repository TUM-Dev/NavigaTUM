use crate::db::public_transport::Transportation;
use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::error;

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
        (status = 200, description = "Things **nearby to the location**", body=NearbyLocationsResponse, content_type = "application/json"),
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
    let public_transport = match Transportation::fetch_all_near(&data.pool, &id).await {
        Ok(public_transport) => public_transport
            .into_iter()
            .map(TransportationResponse::from)
            .collect(),
        Err(e) => {
            error!(error = ?e, "Could not get nearby pois");
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error");
        }
    };
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![
            CacheDirective::MaxAge(2 * 24 * 60 * 60), // valid for 2d
            CacheDirective::Public,
        ]))
        .json(NearbyLocationsResponse { public_transport })
}

#[derive(Serialize, Clone, Debug, utoipa::ToSchema)]
struct NearbyLocationsResponse {
    #[schema(max_items = 50)]
    public_transport: Vec<TransportationResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
struct TransportationResponse {
    /// The globally unique and somewhat stable id of the station from the transport agency
    #[schema(example = "de:09184:2073:0:1")]
    id: String,
    /// How the station was named by the operator
    #[schema(example = "Garching, Boltzmannstraße")]
    name: String,
    /// The globally unique and somewhat stable id of the station from the transport agency
    #[schema(example = "de:09184:2073")]
    parent_id: Option<String>,
    /// How the station was named by the operator
    #[schema(example = "Boltzmannstraße")]
    parent_name: Option<String>,
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
    #[schema(exclusive_minimum = 0.0, exclusive_maximum = 1000.0)]
    distance_meters: f64,
}
impl From<Transportation> for TransportationResponse {
    fn from(value: Transportation) -> Self {
        Self {
            id: value.id,
            name: value.name,
            parent_id: value.parent_id,
            parent_name: value.parent_name,
            lat: value
                .lat
                .expect("since the location is always present, this field can never be null"),
            lon: value
                .lon
                .expect("since the location is always present, this field can never be null"),
            distance_meters: value
                .distance_meters
                .expect("since the location is always present, this field can never be null"),
        }
    }
}
