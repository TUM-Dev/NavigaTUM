use std::fmt;
use std::fmt::Display;
use std::time::Duration;

use cached::proc_macro::io_cached;
use log::{error, warn};

use crate::maps::overlay_map::OverlayMapTask;
use crate::BoxedError;

#[derive(Hash, Debug, Copy, Clone)]
struct TileLocation {
    x: u32,
    y: u32,
    z: u32,
}

impl Display for TileLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TileLocation")
            .field(&self.x)
            .field(&self.y)
            .field(&self.z)
            .finish()
    }
}

#[derive(Debug)]
pub struct FetchTileTask {
    location: TileLocation,
    index: (u32, u32),
}

fn zoom_aware_offset(zoom: u32, value: u32, offset: i32) -> u32 {
    // if we go over the edge of the world, we want to pop in on the other side
    let possible_tiles: i64 = (4_i64).pow(zoom);
    let offset_value = i64::from(value) + i64::from(offset);
    if offset_value < 0 {
        return (possible_tiles + offset_value) as u32;
    }
    (offset_value % possible_tiles) as u32
}

#[cfg(test)]
mod test_tiles {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_zoom_aware_offset() {
        // 1 tile at zoom 0
        assert_eq!(zoom_aware_offset(0, 0, 0), 0);
        assert_eq!(zoom_aware_offset(0, 0, 1), 0);
        // 4 tiles at zoom 1
        assert_eq!(zoom_aware_offset(1, 0, 0), 0);
        assert_eq!(zoom_aware_offset(1, 0, 1), 1);
        assert_eq!(zoom_aware_offset(1, 0, 4), 0);
        assert_eq!(zoom_aware_offset(1, 0, 5), 1);
    }
}

impl FetchTileTask {
    pub fn from(order: &OverlayMapTask) -> Self {
        Self {
            location: TileLocation {
                x: order.x as u32,
                y: order.y as u32,
                z: order.z,
            },
            index: (0, 0),
        }
    }

    pub fn offset_by(self, x_offset: i32, y_offset: i32) -> Self {
        Self {
            location: TileLocation {
                x: zoom_aware_offset(self.location.z, self.location.x, x_offset),
                y: zoom_aware_offset(self.location.z, self.location.y, y_offset),
                z: self.location.z,
            },
            ..self
        }
    }

    pub fn with_index(self, x_index: u32, y_index: u32) -> Self {
        Self {
            index: (x_index, y_index),
            ..self
        }
    }

    // type and create are specified, because a custom conversion is needed
    pub async fn fulfill(self) -> Option<((u32, u32), image::DynamicImage)> {
        let raw_tile = download_map_image(self.location).await;
        match raw_tile {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => Some((self.index, img)),
                Err(e) => {
                    error!("Error while parsing image: {e:#?} for {self:?}");
                    None
                }
            },
            Err(e) => {
                error!("could not fulfill {self:?} because {e}");
                None
            }
        }
    }
}

#[io_cached(disk = true, map_error = r##"|e| format!("{e:?}")"##)]
async fn download_map_image(location: TileLocation) -> Result<Vec<u8>, BoxedError> {
    let url = format!(
        "https://nav.tum.de/maps/styles/osm-liberty/{z}/{x}/{y}@2x.png",
        x = location.x,
        y = location.y,
        z = location.z
    );
    for i in 1..5 {
        let res = reqwest::get(&url).await?.bytes().await?;
        // wait with exponential backoff
        if res.len() > 500 {
            return Ok(res.into());
        }
        let wait_time_ms = 1.5_f32.powi(i).round() as u64;
        let wait_time = Duration::from_millis(wait_time_ms);
        warn!(
            "retrying tileserver-request in {wait_time:?} because it is only {request_len}B",
            request_len = res.len()
        );
        tokio::time::sleep(wait_time).await;
    }
    Err(format!("Got only short Responses from {url}").into())
}
