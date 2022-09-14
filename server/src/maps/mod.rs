use std::io::Cursor;

use actix_web::{get, web, HttpRequest, HttpResponse};
use awc::Client;
use cached::lazy_static::lazy_static;
use cached::proc_macro::cached;
use cached::SizedCache;
use futures::future::join_all;
use image::Rgba;
use imageproc::definitions::HasBlack;
use imageproc::drawing::{draw_text_mut, text_size};
use log::{debug, error, info};
use rusqlite::{Connection, Error, OpenFlags};
use rusttype::{Font, Scale};
use tokio::time::Instant;

use crate::utils;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(maps_handler);
}

struct MapInfo {
    name: String,
    type_common_name: String,
    _type: String,
    lat: f64,
    lon: f64,
}

fn get_localised_data(id: &str, should_use_english: bool) -> Option<Result<MapInfo, Error>> {
    let conn = Connection::open_with_flags(
        "data/api_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("Cannot open database");

    let stmt = conn.prepare_cached(&format!(
        "SELECT name,type,type_common_name,lat,lon FROM {} WHERE key = ?",
        if should_use_english { "en" } else { "de" }
    ));

    let result = match stmt {
        Ok(mut stmt) => stmt.query_row([id], |row| {
            let data = MapInfo {
                name: row.get_unwrap(0),
                _type: row.get_unwrap(1),
                type_common_name: row.get_unwrap(2),
                lat: row.get_unwrap(3),
                lon: row.get_unwrap(4),
            };
            Ok(data)
        }),
        Err(e) => {
            error!("Error preparing statement: {:?}", e);
            return Some(Err(e));
        }
    };
    match result {
        Ok(data) => Some(Ok(data)),
        Err(_) => None,
    }
}

// type and create are specified, because a custom conversion is needed
// size=20 is about 60MB
#[cached(
    type = "SizedCache<String, Vec<u8>>",
    create = "{ SizedCache::with_size(20) }",
    convert = r#"{ _id.to_string() }"#
)]
async fn construct_image_from_data(_id: &str, data: MapInfo) -> Vec<u8> {
    let start_time = Instant::now();
    let mut img = image::RgbaImage::new(1200, 630);

    // add the map
    draw_map(&data, &mut img).await;
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

    wrap_image_in_response(img)
}

fn wrap_image_in_response(img: image::RgbaImage) -> Vec<u8> {
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageOutputFormat::Png).unwrap();
    w.into_inner()
}

async fn download_map_image(z: u32, x: u32, y: u32, file: &str) -> web::Bytes {
    let url = format!(
        "http://localhost:7770/styles/osm_liberty/{}/{}/{}@2x.png",
        z, x, y
    );
    let client = Client::new().get(&url).send();
    let res = match client.await {
        Ok(mut r) => r.body().await.unwrap(),
        Err(e) => {
            error!("failed downloading url {} because {:?}", url, e);
            panic!();
        }
    };
    tokio::fs::write(file, &res)
        .await
        .expect("failed to write file");
    res
}

async fn get_tile(z: u32, x: u32, y: u32, index: (i64, i64)) -> ((i64, i64), image::DynamicImage) {
    // gets the image fro the server. using a disk-cached image if possible
    let file = format!("data/cache/{}_{}_{}@2x.png", z, x, y);
    let file_content = tokio::fs::read(&file).await;
    let tile = match file_content {
        Ok(content) => web::Bytes::from(content),
        Err(_) => download_map_image(z, x, y, &file).await,
    };

    let tile_img = image::load_from_memory(&tile).unwrap();
    (index, tile_img)
}

async fn draw_map(data: &MapInfo, img: &mut image::RgbaImage) {
    let (x, y, z) = lat_lon_to_xyz(data.lat, data.lon);
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
    let (x_img_koords, y_img_koords) = center_to_top_left_coordinates(x_pixels, y_pixels);
    // 3...4*2 entries, because 630-125=505=> max.2 Tiles and 1200=> max 4 tiles
    let mut work_queue = Vec::with_capacity(4 * 2);
    for x_index in 0..5 {
        for y_index in 0..3 {
            if is_in_range(x_img_koords, y_img_koords, x_index, y_index) {
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

    let results: Vec<((i64, i64), image::DynamicImage)> = join_all(work_queue).await;
    for ((x_index, y_index), tile_img) in results {
        image::imageops::overlay(
            img,
            &tile_img,
            x_index * 512 - (x_img_koords as i64),
            y_index * 512 - (y_img_koords as i64),
        );
    }
}

fn center_to_top_left_coordinates(x_pixels: u32, y_pixels: u32) -> (u32, u32) {
    // the center coordniates are usefull for orienting ourselves in one tile,
    // but for drawing them, top left is better
    let y_to_img_border = 512 + y_pixels;
    let y_img_koords = y_to_img_border - (630 - 125) / 2;
    let x_to_img_border = 512 * 2 + x_pixels;
    let x_img_koords = x_to_img_border - 1200 / 2;
    (x_img_koords, y_img_koords)
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

fn lat_lon_to_xyz(lat_deg: f64, lon_deg: f64) -> (f64, f64, u32) {
    let zoom = 16;
    let lat_rad = lat_deg.to_radians();
    let n = 2_u32.pow(zoom) as f64;
    let xtile = (lon_deg + 180.0) / 360.0 * n;
    let ytile = (1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n;
    (xtile, ytile, zoom)
}

lazy_static! {
    static ref CANTARELL_BOLD: Font<'static> =
        Font::try_from_bytes(include_bytes!("font/Cantarell-Bold.ttf")).unwrap();
    static ref CANTARELL_REGULAR: Font<'static> =
        Font::try_from_bytes(include_bytes!("font/Cantarell-Regular.ttf")).unwrap();
}

fn draw_bottom(data: &MapInfo, img: &mut image::RgbaImage) {
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

#[get("/{id}")]
pub async fn maps_handler(
    params: web::Path<String>,
    web::Query(args): web::Query<utils::DetailsQuerryArgs>,
    req: HttpRequest,
) -> HttpResponse {
    let start_time = Instant::now();
    let id = params.into_inner();
    let should_use_english = utils::should_use_english(args, req);
    let data = get_localised_data(&id, should_use_english);
    if data.is_none() {
        return HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found");
    }
    let data = data.unwrap();
    if data.is_err() {
        return HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body("Internal Server Error");
    }
    let res = HttpResponse::Ok()
        .content_type("image/png")
        .body(construct_image_from_data(&id, data.unwrap()).await);

    info!(
        "Preview Generation for {} took {}ms",
        id,
        start_time.elapsed().as_millis()
    );
    res
}
