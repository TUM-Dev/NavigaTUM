use std::io::Cursor;

use actix_web::http::header::LOCATION;
use actix_web::{get, web, HttpResponse};
use image::Rgba;

use log::{debug, error, warn};
use sqlx::PgPool;
use tokio::time::Instant;
use unicode_truncate::UnicodeTruncateStr;

use crate::maps::overlay_map::OverlayMapTask;
use crate::maps::overlay_text::{OverlayText, CANTARELL_BOLD, CANTARELL_REGULAR};
use crate::models::Location;
use crate::models::LocationKeyAlias;
use crate::utils;

mod fetch_tile;
mod overlay_map;
mod overlay_text;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(maps_handler);
    let tile_cache = std::env::temp_dir().join("tiles");
    if !tile_cache.exists() {
        std::fs::create_dir(tile_cache).unwrap();
    }
}

async fn get_localised_data(
    conn: &PgPool,
    id: &str,
    should_use_english: bool,
) -> Result<Location, HttpResponse> {
    let result = if should_use_english {
        sqlx::query_as!(Location, "SELECT * FROM en WHERE key = $1", id)
            .fetch_all(conn)
            .await
    } else {
        sqlx::query_as!(Location, "SELECT * FROM de WHERE key = $1", id)
            .fetch_all(conn)
            .await
    };

    match result {
        Ok(r) => match r.len() {
            0 => Err(HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not found")),
            _ => Ok(r[0].clone()),
        },
        Err(e) => {
            error!("Error preparing statement: {e:?}");
            return Err(HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error"));
        }
    }
}

async fn construct_image_from_data(data: Location) -> Option<Vec<u8>> {
    let start_time = Instant::now();
    let mut img = image::RgbaImage::new(1200, 630);

    // add the map
    if !OverlayMapTask::with(&data).draw_onto(&mut img).await {
        return None;
    }
    debug!("map draw {:?}", start_time.elapsed());

    draw_bottom(&data, &mut img);
    debug!("overlay finish {:?}", start_time.elapsed());
    Some(wrap_image_in_response(&img))
}

fn wrap_image_in_response(img: &image::RgbaImage) -> Vec<u8> {
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageFormat::Png).unwrap();
    w.into_inner()
}

fn draw_bottom(data: &Location, img: &mut image::RgbaImage) {
    // draw background white
    for x in 0..1200 {
        for y in 630 - 125..630 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }
    // add our logo so the bottom
    let logo = image::load_from_memory(include_bytes!("static/logo.png")).unwrap();
    image::imageops::overlay(
        img,
        &logo,
        15,
        630 - (125 / 2) - (i64::from(logo.height()) / 2) + 9,
    );
    let name = if data.name.chars().count() >= 45 {
        format!("{}...", data.name.unicode_truncate(45).0)
    } else {
        data.name.clone()
    };
    OverlayText::with(&name, &CANTARELL_BOLD)
        .at(10, 125 - 10)
        .draw_onto(img);
    OverlayText::with(&data.type_common_name, &CANTARELL_REGULAR)
        .at(10, 125 - 50)
        .draw_onto(img);
}

fn load_default_image() -> Vec<u8> {
    warn!("Loading default preview image, as map rendering failed. Check the connection to the tileserver");
    let img = image::load_from_memory(include_bytes!("static/logo-card.png")).unwrap();
    // encode the image as PNG
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageFormat::Png).unwrap();
    w.into_inner()
}

async fn get_possible_redirect_url(conn: &PgPool, query: &str) -> Option<String> {
    let result = sqlx::query_as!(
        LocationKeyAlias,
        r#"
        SELECT key, visible_id, type
        FROM aliases
        WHERE alias = $1 OR key = $1
        LIMIT 1"#,
        query
    )
    .fetch_one(conn)
    .await;
    match result {
        Ok(d) => Some(format!("https://nav.tum.de/api/preview/{key}", key = d.key)),
        Err(e) => {
            error!("Error requesting alias for {query}: {e:?}");
            None
        }
    }
}

#[get("/{id}")]
pub async fn maps_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<utils::LangQueryArgs>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let start_time = Instant::now();
    let id = params
        .into_inner()
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    if let Some(redirect_url) = get_possible_redirect_url(&data.db, &id).await {
        let mut res = HttpResponse::PermanentRedirect();
        res.insert_header((LOCATION, redirect_url));
        return res.finish();
    }
    let data = match get_localised_data(&data.db, &id, args.should_use_english()).await {
        Ok(data) => data,
        Err(e) => {
            return e.into();
        }
    };
    let img = construct_image_from_data(data)
        .await
        .unwrap_or_else(load_default_image);

    debug!(
        "Preview Generation for {id} took {elapsed:?}",
        elapsed = start_time.elapsed()
    );
    HttpResponse::Ok()
        .content_type("image/png")
        .body(img)
        .into()
}
