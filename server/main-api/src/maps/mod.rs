use std::io::Cursor;

use crate::models::DBRoomEntry;
use actix_web::{get, web, HttpResponse};
use awc::Client;
use cached::lazy_static::lazy_static;
use cached::proc_macro::cached;
use cached::SizedCache;
use diesel::prelude::*;
use futures::future::join_all;
use image::Rgba;
use imageproc::definitions::HasBlack;
use imageproc::drawing::{draw_text_mut, text_size};
use log::{debug, error, warn};
use rusttype::{Font, Scale};
use tokio::time::Instant;

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
// size=20 is about 60MB
#[cached(
    type = "SizedCache<String, Vec<u8>>",
    create = "{ SizedCache::with_size(5) }",
    option = true,
    convert = r#"{ _id.to_string() }"#
)]
async fn construct_image_from_data(_id: &str, data: DBRoomEntry) -> Option<Vec<u8>> {
    let start_time = Instant::now();
    let mut img = image::RgbaImage::new(1200, 630);

    // add the map
    if !draw_map(&data, &mut img).await {
        return None;
    }
    debug!("map draw {}ms", start_time.elapsed().as_millis());

    draw_bottom(&data, &mut img);
    // add the location pin image to the center
    let pin = image::open("src/maps/pin.webp").unwrap();
    image::imageops::overlay(
        &mut img,
        &pin,
        1200 / 2 - pin.width() as i64 / 2,
        (630 - 125) / 2 - pin.height() as i64,
    );
    debug!("overlay finish {}ms", start_time.elapsed().as_millis());
    Some(wrap_image_in_response(img))
}

fn wrap_image_in_response(img: image::RgbaImage) -> Vec<u8> {
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageOutputFormat::Png).unwrap();
    w.into_inner()
}

async fn download_map_image(
    z: u32,
    x: u32,
    y: u32,
    file: &std::path::PathBuf,
) -> Option<web::Bytes> {
    let tileserver_addr =
        std::env::var("MAPS_SVC_PORT_7770_TCP_ADDR").unwrap_or_else(|_| "localhost".to_string());
    let tileserver_port =
        std::env::var("MAPS_SVC_SERVICE_PORT_TILESERVER").unwrap_or_else(|_| "7770".to_string());
    let url =
        format!("http://{tileserver_addr}:{tileserver_port}/styles/osm_liberty/{z}/{x}/{y}@2x.png");
    let client = Client::new().get(&url).send();
    let res = match client.await {
        Ok(mut r) => r.body().await,
        Err(e) => {
            error!("Error downloading map {}: {:?}", url, e);
            return None;
        }
    };
    let res = match res {
        Ok(r) => r,
        Err(e) => {
            error!("Error while payload parsing: {:?}", e);
            return None;
        }
    };

    if let Err(e) = tokio::fs::write(file, &res).await {
        error!(
            "failed to write url {} to {:?} because {:?}. Files wont be cached",
            url, file, e
        );
    };
    Some(res)
}

async fn get_tile(
    z: u32,
    x: u32,
    y: u32,
    index: (i64, i64),
) -> Option<((i64, i64), image::DynamicImage)> {
    // gets the image fro the server. using a disk-cached image if possible
    let file = std::env::temp_dir()
        .join("tiles")
        .join(format!("{z}_{x}_{y}@2x.png"));
    let file_content = tokio::fs::read(&file).await;
    let tile = match file_content {
        Ok(content) => web::Bytes::from(content),
        Err(_) => download_map_image(z, x, y, &file).await?,
    };

    let tile_img = image::load_from_memory(&tile).unwrap();
    Some((index, tile_img))
}

async fn draw_map(data: &DBRoomEntry, img: &mut image::RgbaImage) -> bool {
    let (x, y, z) = entry_to_xyz(data);
    // coordinate system is centered around the center of the image
    // around this center there is a 5*3 grid of tiles
    // -----------------------------------------
    // | -2/ 1 | -1/ 1 |  0/ 1 |  1/ 1 |  2/ 1 |
    // | -2/ 0 | -1/ 0 |   x   |  1/ 0 |  2/ 0 |
    // | -2/-1 | -1/-1 |  0/-1 |  1/-1 |  2/-1 |
    // -----------------------------------------
    // we can now filter for "is on the 1200*630 image" and append them to a work queue

    let x_pixels = (512.0 * (x - x.floor())) as u32;
    let y_pixels = (512.0 * (y - y.floor())) as u32;
    let (x_img_coords, y_img_coords) = center_to_top_left_coordinates(x_pixels, y_pixels);
    // 3...4*2 entries, because 630-125=505=> max.2 Tiles and 1200=> max 4 tiles
    let mut work_queue = Vec::with_capacity(4 * 2);
    for x_index in 0..5 {
        for y_index in 0..3 {
            if is_in_range(x_img_coords, y_img_coords, x_index, y_index) {
                work_queue.push(get_tile(
                    z,
                    x as u32 + x_index - 2,
                    y as u32 + y_index - 1,
                    (x_index as i64, y_index as i64),
                ));
            }
        }
    }
    // the items in the work queue are then asynchronously downloaded and then drawn

    let results: Vec<Option<((i64, i64), image::DynamicImage)>> = join_all(work_queue).await;
    for res in results {
        match res {
            Some(((x_index, y_index), tile_img)) => {
                image::imageops::overlay(
                    img,
                    &tile_img,
                    x_index * 512 - (x_img_coords as i64),
                    y_index * 512 - (y_img_coords as i64),
                );
            }
            None => {
                return false;
            }
        }
    }
    true
}

fn center_to_top_left_coordinates(x_pixels: u32, y_pixels: u32) -> (u32, u32) {
    // the center coordniates are usefull for orienting ourselves in one tile,
    // but for drawing them, top left is better
    let y_to_img_border = 512 + y_pixels;
    let y_img_coords = y_to_img_border - (630 - 125) / 2;
    let x_to_img_border = 512 * 2 + x_pixels;
    let x_img_coords = x_to_img_border - 1200 / 2;
    (x_img_coords, y_img_coords)
}

fn is_in_range(x_pixels: u32, y_pixels: u32, x_index: u32, y_index: u32) -> bool {
    let x_in_range = x_pixels <= (x_index + 1) * 512 && x_pixels + 1200 >= x_index * 512;
    let y_in_range = y_pixels <= (y_index + 1) * 512 && y_pixels + (630 - 125) >= y_index * 512;
    x_in_range && y_in_range
}

#[cfg(test)]
mod range_tests {
    use super::*;

    fn assert_range_eq(
        x_pixels: u32,
        y_pixels: u32,
        expected_x: (u32, u32),
        expected_y: (u32, u32),
    ) {
        for x in 0..10 {
            for y in 0..10 {
                let (x_min, x_max) = expected_x;
                let (y_min, y_max) = expected_y;
                let expected_result = x <= x_max && x >= x_min && y <= y_max && y >= y_min;
                assert_eq!(is_in_range(x_pixels, y_pixels, x, y), expected_result);
            }
        }
    }

    #[test]
    fn ranged_test() {
        assert_range_eq(0, 0, (0, 2), (0, 0));
        assert_range_eq(0, 513, (0, 2), (1, 1));
        assert_range_eq(512 / 2, 0, (0, 2), (0, 0));
        assert_range_eq(512 / 2, 512 / 2, (0, 2), (0, 1));
    }
}

fn entry_to_xyz(entry: &DBRoomEntry) -> (f32, f32, u32) {
    let zoom = match entry.type_.as_str() {
        "campus" => 14,
        "area" | "site" => 15,
        "building" | "joined_building" => 16,
        "virtual_room" | "room" => 17,
        _ => {
            warn!("map generation encountered an type for {entry:?}. Assuming it to be a building");
            16
        }
    };
    lat_lon_z_to_xyz(entry.lat, entry.lon, zoom)
}

fn lat_lon_z_to_xyz(lat_deg: f32, lon_deg: f32, zoom: u32) -> (f32, f32, u32) {
    let lat_rad = lat_deg.to_radians();
    let n = 2_u32.pow(zoom) as f32;
    let xtile = (lon_deg + 180.0) / 360.0 * n;
    let ytile = (1.0 - lat_rad.tan().asinh() / std::f32::consts::PI) / 2.0 * n;
    (xtile, ytile, zoom)
}

lazy_static! {
    static ref CANTARELL_BOLD: Font<'static> =
        Font::try_from_bytes(include_bytes!("font/Cantarell-Bold.ttf")).unwrap();
    static ref CANTARELL_REGULAR: Font<'static> =
        Font::try_from_bytes(include_bytes!("font/Cantarell-Regular.ttf")).unwrap();
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
    // add top text
    let scale = Scale { x: 35.0, y: 35.0 };
    let (w, _) = text_size(scale, &CANTARELL_BOLD, data.name.as_str());
    draw_text_mut(
        img,
        Rgba::black(),
        1200 - w - 10,
        630 - 125 + 10,
        scale,
        &CANTARELL_BOLD,
        data.name.as_str(),
    );
    // add bottom text
    let (w, _) = text_size(scale, &CANTARELL_REGULAR, data.type_common_name.as_str());
    draw_text_mut(
        img,
        Rgba::black(),
        1200 - w - 10,
        630 - 125 + 50,
        scale,
        &CANTARELL_REGULAR,
        data.type_common_name.as_str(),
    );
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
