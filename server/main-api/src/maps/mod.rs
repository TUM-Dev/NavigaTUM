mod fetch_tile;
mod overlay_map;
mod overlay_text;

use std::io::Cursor;

use crate::maps::overlay_map::OverlayMapTask;
use crate::maps::overlay_text::{OverlayText, CANTARELL_BOLD, CANTARELL_REGULAR};
use crate::models::DBRoomEntry;
use actix_web::{get, web, HttpResponse};

use cached::proc_macro::cached;
use cached::SizedCache;
use diesel::prelude::*;
use image::Rgba;

use log::{debug, error, warn};

use tokio::time::Instant;
use unicode_truncate::UnicodeTruncateStr;

use crate::utils;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(maps_handler);
    let tile_cache = std::env::temp_dir().join("tiles");
    if !tile_cache.exists() {
        std::fs::create_dir(tile_cache).unwrap();
    }
}

fn get_localised_data(id: &str, should_use_english: bool) -> Result<DBRoomEntry, HttpResponse> {
    let conn = &mut utils::establish_connection();

    let result = match should_use_english {
        true => {
            use crate::schema::en::dsl::*;
            en.filter(key.eq(&id)).load::<DBRoomEntry>(conn)
        }
        false => {
            use crate::schema::de::dsl::*;
            de.filter(key.eq(&id)).load::<DBRoomEntry>(conn)
        }
    };

    match result {
        Ok(r) => match r.len() {
            0 => Err(HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not found")),
            _ => Ok(r[0].clone()),
        },
        Err(e) => {
            error!("Error preparing statement: {:?}", e);
            return Err(HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error"));
        }
    }
}

// type and create are specified, because a custom conversion is needed
// size=50 is about 150Mi
#[cached(
    type = "SizedCache<String, Vec<u8>>",
    create = "{ SizedCache::with_size(30) }",
    option = true,
    convert = r#"{ _id.to_string() }"#
)]
async fn construct_image_from_data(_id: &str, data: DBRoomEntry) -> Option<Vec<u8>> {
    let start_time = Instant::now();
    let mut img = image::RgbaImage::new(1200, 630);

    // add the map
    if !OverlayMapTask::with(&data).draw_onto(&mut img).await {
        return None;
    }
    debug!("map draw {}ms", start_time.elapsed().as_millis());

    draw_bottom(&data, &mut img);
    debug!("overlay finish {}ms", start_time.elapsed().as_millis());
    Some(wrap_image_in_response(img))
}

fn wrap_image_in_response(img: image::RgbaImage) -> Vec<u8> {
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageOutputFormat::Png).unwrap();
    w.into_inner()
}

fn draw_bottom(data: &DBRoomEntry, img: &mut image::RgbaImage) {
    // draw background white
    for x in 0..1200 {
        for y in 630 - 125..630 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }
    // add our logo so the bottom
    let logo = image::open("src/maps/logo.png").unwrap();
    image::imageops::overlay(
        img,
        &logo,
        15,
        630 - (125 / 2) - (logo.height() as i64 / 2) + 9,
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
    let img = image::open("src/maps/logo-card.png").unwrap();
    // encode the image as PNG
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageOutputFormat::Png).unwrap();
    w.into_inner()
}

#[get("/{id}")]
pub async fn maps_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<utils::LangQueryArgs>,
) -> HttpResponse {
    let start_time = Instant::now();
    let id = params.into_inner();
    let data = match get_localised_data(&id, args.should_use_english()) {
        Ok(data) => data,
        Err(e) => {
            return e;
        }
    };
    let img = construct_image_from_data(&id, data)
        .await
        .unwrap_or_else(load_default_image);
    let res = HttpResponse::Ok().content_type("image/png").body(img);

    debug!(
        "Preview Generation for {id} took {generation_time}ms",
        generation_time = start_time.elapsed().as_millis()
    );
    res
}
