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

impl FetchTileTask {
    pub fn from(z: u32, x: u32, y: u32, index: (u32, u32)) -> Self {
        Self { x, y, z, index }
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

    async fn download_map_image(&self, file: &std::path::PathBuf) -> Option<web::Bytes> {
        let tileserver_addr = std::env::var("MAPS_SVC_PORT_7770_TCP_ADDR")
            .unwrap_or_else(|_| "localhost".to_string());
        let tileserver_port = std::env::var("MAPS_SVC_SERVICE_PORT_TILESERVER")
            .unwrap_or_else(|_| "7770".to_string());
        let url = format!(
            "http://{tileserver_addr}:{tileserver_port}/styles/osm_liberty/{z}/{x}/{y}@2x.png",
            z = self.z,
            x = self.x,
            y = self.y,
        );
        let client = Client::new().get(&url).send();
        let res = match client.await {
            Ok(mut r) => r.body().await,
            Err(e) => {
                error!("Error downloading map {}: {:?}", url, e);
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
