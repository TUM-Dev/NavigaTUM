use actix_web::{get, web, HttpResponse};
use geo_types::Geometry;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tracing::error;

#[tracing::instrument(skip(pool))]
pub async fn fetch_indoor_maps_inside_of(
    pool: &PgPool,
    geom: Geometry,
) -> anyhow::Result<Vec<i64>> {
    let filtered_groups = sqlx::query(
        r#"SELECT group_id
               FROM indoor_features
               WHERE ST_Intersects(convex_hull::geometry, ST_SetSRID($1::geometry,4326))"#,
    )
    .bind(geozero::wkb::Encode(geom))
    .fetch_all(pool)
    .await?;
    let mut filtered_group_ids = Vec::<i64>::new();
    for group in filtered_groups {
        let group_id = group.get_unchecked(0);
        filtered_group_ids.push(group_id);
    }

    Ok(filtered_group_ids)
}
#[tracing::instrument(skip(pool))]
pub async fn fetch_indoor_map(pool: &PgPool, id: i64) -> anyhow::Result<serde_json::Value> {
    let row = sqlx::query("SELECT features from indoor_features where group_id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;
    let value: serde_json::Value = row.get(0);

    Ok(value)
}
#[get("/api/maps/indoor/{id}")]
pub async fn get_indoor_map(
    params: web::Path<i64>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params.into_inner();
    let map = fetch_indoor_map(&data.pool, id).await;
    match map {
        Ok(geometry) => HttpResponse::Ok().json(geometry),
        Err(err) => {
            error!("Failed to fetch indoor map {id} because {err:?}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize)]
struct RemoteMap {
    name: String,
    url: Url,
}

#[derive(Deserialize)]
struct Arguments {
    bbox: String,
}
impl Arguments {
    fn validate_bbox(&self) -> Result<geo::Rect<f64>, HttpResponse> {
        let bbox: Vec<f64> = self
            .bbox
            .split(",")
            .filter_map(|s| s.parse().ok())
            .collect();
        if bbox.len() != 4 {
            return Err(HttpResponse::BadRequest()
                .body("the bbox-parameter needs 4 floading point numbers with"));
        }
        Ok(geo::Rect::new(
            geo::Coord::from((bbox[1], bbox[0])),
            geo::Coord::from((bbox[3], bbox[2])),
        ))
    }
}

#[get("/api/maps/indoor")]
pub async fn list_indoor_maps(
    web::Query(args): web::Query<Arguments>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let bbox = match args.validate_bbox() {
        Ok(bbox) => bbox,
        Err(e) => return e,
    };
    let maps = fetch_indoor_maps_inside_of(&data.pool, bbox.into()).await;
    let maps = match maps {
        Ok(m) => m,
        Err(e) => {
            error!("could not list maps because {e:?}");
            return HttpResponse::InternalServerError()
                .body("could not get indoor maps, please try again later");
        }
    };
    let mut response = Vec::new();
    for map in maps {
        response.push(RemoteMap {
            name: map.to_string(),
            url: format!("https://nav.tum.de/api/maps/indoor/{map}")
                .parse()
                .unwrap(),
        })
    }

    HttpResponse::Ok().json(response)
}
