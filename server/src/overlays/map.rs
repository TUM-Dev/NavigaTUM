use image::imageops;
use tracing::warn;

use crate::external::static_map_image::{Camera, download_static_map_image};

const BOTTOM_PANEL_HEIGHT: u32 = 125;

#[derive(Debug)]
pub struct OverlayMapTask {
    camera: Camera,
}

impl OverlayMapTask {
    pub fn new(r#type: &str, lat: f64, lon: f64) -> Self {
        // tilt buildings: they render at a zoom where the basemap still has 3D extrusions.
        let (zoom, pitch) = match r#type {
            "campus" => (14, 0),
            "area" | "site" => (15, 0),
            "building" | "joined_building" => (16, 20),
            "virtual_room" | "room" | "poi" => (17, 0),
            entry => {
                warn!(
                    ?entry,
                    "map generation encountered an unknown type, assuming it to be a building"
                );
                (16, 20)
            }
        };
        Self {
            camera: Camera {
                lat,
                lon,
                zoom,
                pitch,
            },
        }
    }

    #[tracing::instrument(skip(img))]
    pub async fn draw_onto(&self, img: &mut image::RgbaImage) -> bool {
        let map_height = img.height() - BOTTOM_PANEL_HEIGHT;
        match download_static_map_image(self.camera, img.width(), map_height).await {
            Ok(map) => {
                imageops::overlay(img, &map, 0, 0);
                true
            }
            Err(e) => {
                warn!(error = ?e, camera = ?self.camera, "could not render preview basemap");
                false
            }
        }
    }
}
