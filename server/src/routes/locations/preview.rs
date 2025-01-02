use std::io::Cursor;

use actix_web::http::header::{CacheControl, CacheDirective, LOCATION};
use actix_web::{get, web, HttpResponse};
use image::{ImageBuffer, Rgba};
use serde::Deserialize;
use sqlx::Error::RowNotFound;
use sqlx::PgPool;
use tracing::{error, warn};
use unicode_truncate::UnicodeTruncateStr;

use crate::limited::vec::LimitedVec;
use crate::localisation;
use crate::overlays::map::OverlayMapTask;
use crate::overlays::text::{cantarell_bold, cantarell_regular, OverlayText};

#[derive(Debug, Clone)]
#[allow(dead_code)] // false positive. Clippy can't detect this due to macros
pub struct LocationKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}

#[derive(Debug)]
struct Location {
    name: String,
    r#type: String,
    type_common_name: String,
    lat: f64,
    lon: f64,
}

#[tracing::instrument(skip(pool))]
async fn get_localised_data(
    pool: &PgPool,
    id: &str,
    should_use_english: bool,
) -> Result<Location, HttpResponse> {
    let result = if should_use_english {
        sqlx::query_as!(
            Location,
            "SELECT type,lat,lon,name,type_common_name FROM en WHERE key = $1",
            id
        )
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as!(
            Location,
            "SELECT type,lat,lon,name,type_common_name FROM de WHERE key = $1",
            id
        )
        .fetch_all(pool)
        .await
    };

    match result {
        Ok(mut r) => match r.pop() {
            None => Err(HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not found")),
            Some(item) => Ok(item),
        },
        Err(e) => {
            error!("Error preparing statement: {e:?}");
            return Err(HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Could not get data for location, please try again later"));
        }
    }
}

#[tracing::instrument]
async fn construct_image_from_data(
    data: Location,
    format: PreviewFormat,
) -> Option<LimitedVec<u8>> {
    let mut img = match format {
        PreviewFormat::OpenGraph => image::RgbaImage::new(1200, 630),
        PreviewFormat::Square => image::RgbaImage::new(1200, 1200),
    };

    // add the map
    if !OverlayMapTask::new(&data.r#type, data.lat, data.lon)
        .draw_onto(&mut img)
        .await
    {
        return None;
    }
    draw_pin(&mut img);

    draw_bottom(&data, &mut img);
    Some(wrap_image_in_response(&img))
}

/// add the location pin image to the center
#[tracing::instrument(skip(img),level = tracing::Level::DEBUG, )]
fn draw_pin(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let pin = image::load_from_memory(include_bytes!("static/pin.png")).unwrap();
    image::imageops::overlay(
        img,
        &pin,
        (img.width() as i64) / 2 - i64::from(pin.width()) / 2,
        ((img.height() as i64) - 125) / 2 - i64::from(pin.height()),
    );
}

fn wrap_image_in_response(img: &image::RgbaImage) -> LimitedVec<u8> {
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageFormat::Png).unwrap();
    LimitedVec(w.into_inner())
}
const WHITE_PIXEL: Rgba<u8> = Rgba([255, 255, 255, 255]);

#[tracing::instrument(skip(img),level = tracing::Level::DEBUG)]
fn draw_bottom(data: &Location, img: &mut image::RgbaImage) {
    // draw background white
    for x in 0..img.width() {
        for y in img.height() - 125..img.height() {
            img.put_pixel(x, y, WHITE_PIXEL);
        }
    }
    // add our logo so the bottom
    let logo = image::load_from_memory(include_bytes!("static/logo.png")).unwrap();
    image::imageops::overlay(
        img,
        &logo,
        15,
        img.height() as i64 - (125 / 2) - (i64::from(logo.height()) / 2) + 9,
    );
    let name = if data.name.chars().count() >= 45 {
        format!("{}...", data.name.unicode_truncate(45).0)
    } else {
        data.name.clone()
    };
    OverlayText::with(&name, cantarell_bold())
        .at(10, 125 - 10)
        .draw_onto(img);
    OverlayText::with(&data.type_common_name, cantarell_regular())
        .at(10, 125 - 50)
        .draw_onto(img);
}

fn load_default_image() -> LimitedVec<u8> {
    warn!("Loading default preview image, as map rendering failed. Check the connection to the tileserver");
    let img = image::load_from_memory(include_bytes!("static/logo-card.png")).unwrap();
    // encode the image as PNG
    let mut w = Cursor::new(Vec::new());
    img.write_to(&mut w, image::ImageFormat::Png).unwrap();
    LimitedVec(w.into_inner())
}

#[tracing::instrument(skip(pool))]
async fn get_possible_redirect_url(pool: &PgPool, query: &str, args: &QueryArgs) -> Option<String> {
    let result = sqlx::query_as!(
        LocationKeyAlias,
        r#"
        SELECT key, visible_id, type
        FROM aliases
        WHERE alias = $1 AND key <> alias
        LIMIT 1"#,
        query
    )
    .fetch_one(pool)
    .await;
    match result {
        Ok(d) => Some(format!(
            "https://nav.tum.de/api/locations/{key}/preview?lang={lang}&format={format}",
            key = d.key,
            lang = args.lang.serialise(),
            format = args.format.serialise()
        )),
        Err(RowNotFound) => None,
        Err(e) => {
            error!("Error requesting alias for {query}: {e:?}");
            None
        }
    }
}

#[derive(Deserialize, Default, Debug, Copy, Clone, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum PreviewFormat {
    #[default]
    OpenGraph,
    Square,
}
impl PreviewFormat {
    fn serialise(self) -> String {
        match self {
            PreviewFormat::OpenGraph => "open_graph".to_string(),
            PreviewFormat::Square => "square".to_string(),
        }
    }
}

#[derive(Deserialize, Default, Debug, utoipa::IntoParams)]
struct QueryArgs {
    #[serde(flatten)]
    lang: localisation::LangQueryArgs,
    format: PreviewFormat,
}

#[derive(Deserialize, utoipa::IntoParams)]
struct MapsPathParams {
    id: String,
}

/// Get a entry-preview
///
/// This returns a 1200x630px preview for the location (room/building/..).
///
/// This is usefully for implementing custom OpenGraph images for detail previews.
#[utoipa::path(
    tags=["locations"],
    params(MapsPathParams, QueryArgs),
    responses(
        (status = 200, description = "**Preview image**", content_type="image/png"),
        (status = 404, description = "**Not found.** Make sure that requested item exists", body = String, content_type = "text/plain", example = "Not found"),
    )
)]
#[get("/api/locations/{id}/preview")]
pub async fn maps_handler(
    params: web::Path<MapsPathParams>,
    args: web::Query<QueryArgs>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params
        .id
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    if let Some(redirect_url) = get_possible_redirect_url(&data.pool, &id, &args).await {
        return HttpResponse::PermanentRedirect()
            .insert_header((LOCATION, redirect_url))
            .finish();
    }
    let data = match get_localised_data(&data.pool, &id, args.lang.should_use_english()).await {
        Ok(data) => data,
        Err(e) => {
            return e;
        }
    };
    let img = construct_image_from_data(data, args.format)
        .await
        .unwrap_or_else(load_default_image);
    HttpResponse::Ok()
        .content_type("image/png")
        .insert_header(CacheControl(vec![
            CacheDirective::MaxAge(2 * 24 * 60 * 60), // valid for 2d
            CacheDirective::Public,
        ]))
        .body(img.0)
}
