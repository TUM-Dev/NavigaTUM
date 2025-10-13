use crate::db::location::LocationKeyAlias;
use crate::limited::vec::LimitedVec;
use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{HttpResponse, get, web};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fast_qr::convert::{Builder, Shape, image::ImageBuilder};
use fast_qr::qr::QRBuilder;
use serde::Deserialize;
use tracing::error;

#[derive(Deserialize, utoipa::IntoParams)]
struct QrCodePathParams {
    id: String,
}

#[tracing::instrument]
fn generate_qr_code(url: &str) -> anyhow::Result<LimitedVec<u8>> {
    // Build the QR code
    let qrcode = QRBuilder::new(url)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build QR code: {e}"))?;

    // Generate the image with rounded corners
    let pin = include_bytes!("static/pin.png");
    let pin_base64 = BASE64_STANDARD.encode(pin);
    let png_code = ImageBuilder::default()
        .margin(1)
        .shape(Shape::RoundedSquare)
        .module_color("#3070B3")
        .background_color("#ffffff")
        .fit_width(600)
        .fit_height(600)
        .image(format!("data:image/png;base64,{pin_base64}"))
        .to_bytes(&qrcode)
        .map_err(|e| anyhow::anyhow!("cannot build QR code: {e}"))?;

    Ok(LimitedVec(png_code))
}

/// Get a QR code for a location
///
/// This returns a QR code image (PNG) that links to the location's detail page.
/// The QR code uses TUM blue (#0065bd) as foreground color with white background and rounded corners.
#[utoipa::path(
    tags=["locations"],
    params(QrCodePathParams),
    responses(
        (status = 200, description = "**QR code image**", content_type="image/png"),
        (status = 400, description = "**Bad request.** Make sure that requested item ID is not empty and not longer than 255 characters", body = String, content_type = "text/plain", example = "Invalid ID"),
        (status = 404, description = "**Not found.** Make sure that requested item exists", body = String, content_type = "text/plain", example = "Not found"),
        (status = 500, description = "**Internal server error**", body = String, content_type = "text/plain"),
    )
)]
#[get("/api/locations/{id}/qr-code")]
pub async fn qr_code_handler(
    params: web::Path<QrCodePathParams>,
    data: web::Data<crate::AppData>,
) -> HttpResponse {
    let id = params
        .id
        .replace(|c: char| c.is_whitespace() || c.is_control(), "");
    if params.id.is_empty() || params.id.len() > 255 {
        return HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("Invalid ID");
    }

    let id = match LocationKeyAlias::fetch_optional(&data.pool, &id).await {
        Ok(Some(id)) => format!("https://nav.tum.de/{type}/{id}", type = id.r#type
          , id = id.key),
        Ok(None) => {
            return HttpResponse::NotFound()
                .content_type("text/plain")
                .body("Not found");
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch location key alias. Assuming it is legitimate, since the generated links are a 404 in the worst case");
            format!("https://nav.tum.de/view/{id}")
        }
    };

    // Location exists, generate QR code
    match generate_qr_code(&id) {
        Ok(qr_image) => HttpResponse::Ok()
            .content_type("image/png")
            .insert_header(CacheControl(vec![
                CacheDirective::MaxAge(7 * 24 * 60 * 60), // valid for 7d
                CacheDirective::Public,
            ]))
            .body(qr_image.0),
        Err(e) => {
            error!(error = %e, "Failed to generate QR code");
            HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Failed to generate QR code")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_code() {
        let result = generate_qr_code("https://nav.tum.de/view/5510.00.001");

        assert!(result.is_ok(), "QR code generation should succeed");
        let qr_image = result.unwrap();

        // Verify it's a valid PNG
        assert!(!qr_image.0.is_empty(), "QR code should not be empty");

        // PNG files start with these magic bytes
        assert_eq!(
            &qr_image.0[0..8],
            &[137, 80, 78, 71, 13, 10, 26, 10],
            "Should be a valid PNG file"
        );

        // Use insta binary snapshot to verify the exact output
        insta::assert_binary_snapshot!("qr_code_5510_00_001.png", qr_image.0);
    }

    #[test]
    /// Tests that the same ID always generates the same QR code
    fn test_qr_code_consistency() {
        let url = "https://nav.tum.de/view/mi";
        let result1 = generate_qr_code(url).unwrap();
        let result2 = generate_qr_code(url).unwrap();

        assert_eq!(
            result1.0, result2.0,
            "QR code generation should be deterministic"
        );
    }

    #[test]
    /// Test that different IDs generate different QR codes
    fn test_qr_code_different_ids() {
        let result1 = generate_qr_code("https://nav.tum.de/view/5510.01.001").unwrap();
        let result2 = generate_qr_code("https://nav.tum.de/view/5602.EG.001").unwrap();

        assert_ne!(
            result1.0, result2.0,
            "Different IDs should generate different QR codes"
        );
    }
}
