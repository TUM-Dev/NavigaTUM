use crate::maps::overlay::OverlayMapTask;
use actix_web::web;
use awc::Client;
use log::error;

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
        let file = std::env::temp_dir().join("tiles").join(filename);
        let file_content = tokio::fs::read(&file).await;
        let tile = match file_content {
            Ok(content) => web::Bytes::from(content),
            Err(_) => self.download_map_image(&file).await?,
        };

        let tile_img = image::load_from_memory(&tile);
        match tile_img {
            Ok(img) => Some((self.index, img)),
            Err(e) => {
                error!("Error while parsing image: {e:#?} for {file:?}");
                None
            }
        }
    }

    fn get_tileserver_url(&self) -> String {
        let tileserver_addr = std::env::var("MAPS_SVC_PORT_7770_TCP_ADDR");
        let tileserver_port = std::env::var("MAPS_SVC_SERVICE_PORT_TILESERVER");
        let base_url = match (tileserver_port, tileserver_addr) {
            (Ok(port), Ok(addr)) => format!("http://{port}:{addr}"),
            _ => "https://nav.tum.de/maps".to_string(),
        };
        format!(
            "{base_url}/styles/osm_liberty/{z}/{x}/{y}@2x.png",
            z = self.z,
            x = self.x,
            y = self.y,
        )
    }

    async fn download_map_image(&self, file: &std::path::PathBuf) -> Option<web::Bytes> {
        let url = self.get_tileserver_url();
        let client = Client::new().get(&url).send();
        let res = match client.await {
            Ok(mut r) => r.body().await,
            Err(e) => {
                error!("Error downloading map {url}: {e:?}");
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
}
