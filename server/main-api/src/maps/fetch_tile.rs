use crate::maps::overlay_map::OverlayMapTask;
use actix_web::web;
use log::{error, warn};
use std::io;
use std::io::ErrorKind;

pub(crate) struct FetchTileTask {
    x: u32,
    y: u32,
    z: u32,
    index: (u32, u32),
}

fn zoom_aware_offset(zoom: u32, value: u32, offset: i32) -> u32 {
    // if we go over the edge of the world, we want to pop in on the other side
    let possible_tiles: i64 = (4_i64).pow(zoom);
    let offset_value = value as i64 + offset as i64;
    if offset_value < 0 {
        return (possible_tiles + offset_value) as u32;
    }
    (offset_value % possible_tiles) as u32
}
#[cfg(test)]
mod test_tiles {
    use super::*;
    use pretty_assertions::assert_eq;

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
            x: order.x as u32,
            y: order.y as u32,
            z: order.z,
            index: (0, 0),
        }
    }

    pub(crate) fn offset_by(self, x_offset: i32, y_offset: i32) -> Self {
        Self {
            x: zoom_aware_offset(self.z, self.x, x_offset),
            y: zoom_aware_offset(self.z, self.y, y_offset),
            ..self
        }
    }

    pub fn with_index(self, x_index: u32, y_index: u32) -> Self {
        Self {
            index: (x_index, y_index),
            ..self
        }
    }

    pub async fn fulfill(self) -> Option<((u32, u32), image::DynamicImage)> {
        // gets the image fro the server. using a disk-cached image if possible
        let filename = format!("{}_{}_{}@2x.png", self.z, self.x, self.y);
        let file_path = std::env::temp_dir().join("tiles").join(filename);
        let tile = match tokio::fs::read(&file_path).await {
            Ok(content) => web::Bytes::from(content),
            Err(_) => {
                let mut tile = self.download_map_image(&file_path).await;
                for i in 1..3 {
                    if tile.is_err() {
                        warn!("Error while downloading {file_path:?} {i} times. Retrying");
                        tile = self.download_map_image(&file_path).await;
                    }
                }
                match tile {
                    Ok(t) => t,
                    Err(e) => {
                        error!(
                            "could not fulfill {file_path:?} 3 times. Giving up. Last error {e:?}"
                        );
                        return None;
                    }
                }
            }
        };

        match image::load_from_memory(&tile) {
            Ok(img) => Some((self.index, img)),
            Err(e) => {
                error!("Error while parsing image: {e:#?} for {file_path:?}");
                None
            }
        }
    }

    fn get_tileserver_url(&self) -> String {
        format!(
            "https://nav.tum.de/maps/styles/osm_liberty/{z}/{x}/{y}@2x.png",
            z = self.z,
            x = self.x,
            y = self.y,
        )
    }
    async fn download_map_image(
        &self,
        file: &std::path::PathBuf,
    ) -> Result<web::Bytes, Box<dyn std::error::Error>> {
        let url = self.get_tileserver_url();
        let res = reqwest::get(&url).await?.bytes().await?;

        if let response_size @ 0..=500 = res.len() {
            return Err(io::Error::new(
                ErrorKind::Other,
                format!("Got a short Response from {url}. . Response ({response_size}B): {res:?}"),
            )
            .into());
        }

        if let Err(e) = tokio::fs::write(file, &res).await {
            warn!("failed to write {url} to {file:?} because {e:?}. Files wont be cached");
        };
        Ok(res)
    }
}
