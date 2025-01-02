use std::fmt::Display;
use std::time::Duration;
use std::{fmt, io};

use tracing::{error, warn};

use crate::limited::vec::LimitedVec;
use crate::maps::overlay_map::OverlayMapTask;

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

#[derive(Debug, Hash, Copy, Clone)]
pub struct MapImageDownloadTask {
    location: TileLocation,
    index: (u32, u32),
}

impl From<&OverlayMapTask> for MapImageDownloadTask {
    fn from(overlay: &OverlayMapTask) -> Self {
        Self {
            location: TileLocation {
                x: overlay.x as u32,
                y: overlay.y as u32,
                z: overlay.z,
            },
            index: (0, 0),
        }
    }
}

impl MapImageDownloadTask {
    /// if we go over the edge of the world, we want to pop in on the other side
    /// unsure if this edge-case is worth covering in more depth
    /// occurs because we naively take the covered tiles without taking the wrapping into account
    fn offset_zoom_aware(zoom: u32, value: u32, offset: i32) -> u32 {
        let possible_tiles: i64 = (4_i64).pow(zoom);
        let offset_value = i64::from(value) + i64::from(offset);
        if offset_value < 0 {
            return (possible_tiles + offset_value) as u32;
        }
        (offset_value % possible_tiles) as u32
    }
    pub fn offset_by(self, x_offset: i32, y_offset: i32) -> Self {
        Self {
            location: TileLocation {
                x: Self::offset_zoom_aware(self.location.z, self.location.x, x_offset),
                y: Self::offset_zoom_aware(self.location.z, self.location.y, y_offset),
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
    #[tracing::instrument(ret(level = tracing::Level::TRACE))]
    pub async fn fulfill(self) -> Option<((u32, u32), image::DynamicImage)> {
        let raw_tile = download_map_image(self.location).await;
        match raw_tile {
            Ok(bytes) => match image::load_from_memory(&bytes.0) {
                Ok(img) => Some((self.index, img)),
                Err(e) => {
                    error!("Error while parsing image: {e:?} for {self:?}");
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

#[tracing::instrument]
async fn download_map_image(location: TileLocation) -> anyhow::Result<LimitedVec<u8>> {
    let url = format!(
        "https://nav.tum.de/tiles/render/navigatum-basemap/{z}/{x}/{y}@2x.png",
        x = location.x,
        y = location.y,
        z = location.z
    );
    for i in 1..5 {
        let response = reqwest::get(&url).await?;
        let status = response.status();
        if status.as_u16() == 400 {
            error!("could not find {location:?} at {url} with {status:?}");
            return Err(io::Error::other("could not find requested tile").into());
        }
        let bytes = response.bytes().await?;
        // wait with exponential backoff
        let size = bytes.len();
        if size > 500 {
            return Ok(LimitedVec(bytes.into()));
        }
        let wait_time_ms = 1.5_f32.powi(i).round() as u64;
        let wait_time = Duration::from_millis(wait_time_ms);
        warn!("retrying {url} in {wait_time:?} because response({status:?}) is only {size}B");
        tokio::time::sleep(wait_time).await;
    }
    Err(anyhow::anyhow!("Got only short Responses from {url}"))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    /// Zoom 0 has 1 tile
    fn test_zoom_aware_offset_z0() {
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(0, 0, -1), 0);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(0, 0, 0), 0);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(0, 0, 1), 0);
    }

    #[test]
    /// Zoom 1 has 4 tiles
    fn test_zoom_aware_offset_z1() {
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(1, 0, -1), 3);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(1, 0, 0), 0);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(1, 0, 1), 1);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(1, 0, 2), 2);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(1, 0, 3), 3);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(1, 0, 4), 0);
        assert_eq!(MapImageDownloadTask::offset_zoom_aware(1, 0, 5), 1);
    }
}
