use std::io;
use std::time::Duration;

use image::{DynamicImage, load_from_memory};
use tokio::time::sleep;
use tracing::{error, warn};

/// Camera for Martin's static-image renderer, centered on a coordinate and north up.
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub lat: f64,
    pub lon: f64,
    pub zoom: u32,
    /// Tilt in degrees (`0` looks straight down).
    pub pitch: u32,
}

/// Endpoint: `GET /style/{id}/static/{lon},{lat},{zoom}[@{bearing},{pitch}]/{w}x{h}.{ext}`.
fn static_image_url(camera: Camera, width: u32, height: u32) -> String {
    let Camera {
        lon,
        lat,
        zoom,
        pitch,
    } = camera;
    let view = if pitch == 0 {
        format!("{lon},{lat},{zoom}")
    } else {
        format!("{lon},{lat},{zoom}@0,{pitch}")
    };
    format!("https://nav.tum.de/martin/style/navigatum-basemap/static/{view}/{width}x{height}.png")
}

/// Render a basemap image via Martin's static-image API, retrying transient `5xx`
/// responses (static rendering has no concurrency support) with exponential backoff.
#[tracing::instrument]
pub async fn download_static_map_image(
    camera: Camera,
    width: u32,
    height: u32,
) -> anyhow::Result<DynamicImage> {
    let url = static_image_url(camera, width, height);
    for i in 1..5 {
        let response = reqwest::get(&url).await?;
        let status = response.status();
        if status.is_success() {
            let bytes = response.bytes().await?;
            return Ok(load_from_memory(&bytes)?);
        }
        if status.is_client_error() {
            error!(url, ?status, "static map render rejected the request");
            return Err(io::Error::other("static map render rejected the request").into());
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "backoff is a small positive value"
        )]
        let wait_time = Duration::from_millis(1.5_f32.powi(i).round() as u64);
        warn!(url, ?status, retrying_in = ?wait_time, "retrying static map render");
        sleep(wait_time).await;
    }
    Err(anyhow::anyhow!("could not render static map from {url}"))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_static_image_url_flat() {
        let camera = Camera {
            lat: 48.262_5,
            lon: 11.668_8,
            zoom: 16,
            pitch: 0,
        };
        assert_eq!(
            static_image_url(camera, 1200, 505),
            "https://nav.tum.de/martin/style/navigatum-basemap/static/11.6688,48.2625,16/1200x505.png"
        );
    }

    #[test]
    fn test_static_image_url_tilted() {
        let camera = Camera {
            lat: 48.262_5,
            lon: 11.668_8,
            zoom: 16,
            pitch: 20,
        };
        assert_eq!(
            static_image_url(camera, 1200, 505),
            "https://nav.tum.de/martin/style/navigatum-basemap/static/11.6688,48.2625,16@0,20/1200x505.png"
        );
    }
}
