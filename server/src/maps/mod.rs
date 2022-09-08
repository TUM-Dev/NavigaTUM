use std::io::Cursor;

use actix_web::{get, web, HttpRequest, HttpResponse};
use awc::Client;
use cached::lazy_static::lazy_static;
use log::{error, info};
use rusqlite::{Connection, Error, OpenFlags};

use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};

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

fn get_localised_data(id: String, should_use_english: bool) -> Option<Result<MapInfo, Error>> {
    let conn = Connection::open_with_flags(
        "data/api_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("Cannot open database");

    let stmt =
        match should_use_english {
            false => conn
                .prepare_cached("SELECT name,type,type_common_name,lat,lon FROM de WHERE key = ?"),
            true => conn
                .prepare_cached("SELECT name,type,type_common_name,lat,lon FROM en WHERE key = ?"),
        };

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

async fn construct_image_from_data(data: MapInfo) -> HttpResponse {
    let mut img = image::RgbaImage::new(1200, 630);

    // add the map
    draw_map(&data, &mut img).await;

    draw_bottom(&data, &mut img);
    // add the location pin image to the center
    let logo = image::open("src/maps/pin.webp").unwrap();
    image::imageops::overlay(
        &mut img,
        &logo,
        1200 / 2 - logo.width() as i64 / 2,
        (630 - 150) / 2 - logo.height() as i64 / 2,
    );

    let mut w = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut w, image::ImageOutputFormat::Png)
        .unwrap();
    let vec = w.into_inner();
    HttpResponse::Ok().content_type("image/png").body(vec)
}

async fn download_map_image(z: i32, x: i32, y: i32, file: &str) -> web::Bytes {
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
async fn get_map_image(z: i32, x: i32, y: i32) -> web::Bytes {
    // gets the image fro the server. using a disk-cached image if possible
    let file = format!("data/cache/{}_{}_{}@2x.png", z, x, y);
    let file_content = tokio::fs::read(&file).await;
    match file_content {
        Ok(content) => web::Bytes::from(content),
        Err(_) => download_map_image(z, x, y, &file).await,
    }
}

async fn draw_map(data: &MapInfo, img: &mut image::RgbaImage) {
    let (x, y, z) = lat_lon_to_xyz(data.lat, data.lon);
    let data = get_map_image(z, x, y).await;
    let map = image::load_from_memory(&data).unwrap();
    image::imageops::overlay(img, &map, 0, 0);
}

fn lat_lon_to_xyz(lat_deg: f64, lon_deg: f64) -> (i32, i32, i32) {
    let zoom = 16_i32;
    let lat_rad = lat_deg.to_radians();
    let n = 2.0_f64.powi(zoom);
    let xtile = (lon_deg + 180.0) / 360.0 * n;
    let ytile = (1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n;
    (xtile as i32, ytile as i32, zoom)
}

lazy_static! {
    static ref CANTARELL_BOLD: Font<'static> =
        Font::try_from_vec(Vec::from(include_bytes!("font/Cantarell-Bold.ttf") as &[u8])).unwrap();
    static ref CANTARELL_REGULAR: Font<'static> = Font::try_from_vec(Vec::from(include_bytes!(
        "font/Cantarell-Regular.ttf"
    ) as &[u8]))
    .unwrap();
}

fn draw_bottom(data: &MapInfo, img: &mut image::RgbaImage) {
    // draw background white
    for x in 0..1200 {
        for y in 630 - 150..630 {
            img.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        }
    }
    // add our logo so the bottom
    let logo = image::open("src/maps/logo.png").unwrap();
    //let logo_height = logo_width / 200.832 * 32.115;
    image::imageops::overlay(img, &logo, 10, 630 - (150 / 2) - (logo.height() as i64 / 2));
    // add top text
    let scale = Scale { x: 35.0, y: 35.0 };
    let color_black = image::Rgba([0, 0, 0, 255]);
    let (w, _) = text_size(scale, &CANTARELL_BOLD, data.name.as_str());
    draw_text_mut(
        img,
        color_black,
        1200 - w - 10,
        630 - 150 + 10,
        scale,
        &CANTARELL_BOLD,
        data.name.as_str(),
    );
    // add bottom text
    let font = Vec::from(include_bytes!("font/Cantarell-Regular.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let (w, _) = text_size(scale, &font, data.type_common_name.as_str());
    draw_text_mut(
        img,
        color_black,
        1200 - w - 10,
        630 - 150 + 50,
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
    let id = params.into_inner();
    let should_use_english = utils::should_use_english(args, req);
    let data = get_localised_data(id, should_use_english);
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
    construct_image_from_data(data.unwrap()).await
}
