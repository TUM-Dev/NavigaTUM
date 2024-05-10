use futures::{stream::FuturesUnordered, StreamExt};
use log::warn;

use crate::maps::fetch_tile::FetchTileTask;
use crate::models::Location;

pub struct OverlayMapTask {
    pub x: f64,
    pub y: f64,
    pub z: u32,
}

impl OverlayMapTask {
    pub fn with(entry: &Location) -> Self {
        let zoom = match entry.r#type.as_str() {
            "campus" => 14,
            "area" | "site" => 15,
            "building" | "joined_building" => 16,
            "virtual_room" | "room" | "poi" => 17,
            _ => {
                warn!("map generation encountered an type for {entry:?}. Assuming it to be a building");
                16
            }
        };
        let (x, y, z) = lat_lon_z_to_xyz(entry.lat, entry.lon, zoom);
        Self { x, y, z }
    }
    pub async fn draw_onto(&self, img: &mut image::RgbaImage) -> bool {
        // coordinate system is centered around the center of the image
        // around this center there is a 5*3 grid of tiles
        // -----------------------------------------
        // | -2/ 1 | -1/ 1 |  0/ 1 |  1/ 1 |  2/ 1 |
        // | -2/ 0 | -1/ 0 |   x   |  1/ 0 |  2/ 0 |
        // | -2/-1 | -1/-1 |  0/-1 |  1/-1 |  2/-1 |
        // -----------------------------------------
        // we can now filter for "is on the 1200*630 image" and append them to a work queue

        let x_pixels = (512.0 * (self.x - self.x.floor())) as u32;
        let y_pixels = (512.0 * (self.y - self.y.floor())) as u32;
        let (x_img_coords, y_img_coords) = center_to_top_left_coordinates(x_pixels, y_pixels);
        // the queue can have 3...4*2 entries, because 630-125=505=> max.2 Tiles and 1200=> max 4 tiles
        let mut work_queue = FuturesUnordered::new();
        for x_index in 0..5 {
            for y_index in 0..3 {
                if is_in_range(x_img_coords, y_img_coords, x_index, y_index) {
                    work_queue.push(
                        FetchTileTask::from(self)
                            .offset_by((x_index as i32) - 2, (y_index as i32) - 1)
                            .with_index(x_index, y_index)
                            .fulfill(),
                    );
                }
            }
        }
        // draw the tiles onto the image after receiving them
        while let Some(res) = work_queue.next().await {
            match res {
                Some(((x_index, y_index), tile_img)) => {
                    let x = x_index as i64 * 512 - (x_img_coords as i64);
                    let y = y_index as i64 * 512 - (y_img_coords as i64);
                    image::imageops::overlay(img, &tile_img, x, y);
                }
                None => {
                    return false;
                }
            }
        }

        // add the location pin image to the center
        let pin = image::load_from_memory(include_bytes!("static/pin.webp")).unwrap();
        image::imageops::overlay(
            img,
            &pin,
            1200 / 2 - i64::from(pin.width()) / 2,
            (630 - 125) / 2 - i64::from(pin.height()),
        );
        true
    }
}

fn lat_lon_z_to_xyz(lat_deg: f64, lon_deg: f64, zoom: u32) -> (f64, f64, u32) {
    let lat_rad = lat_deg.to_radians();
    let n = 2_u32.pow(zoom) as f64;
    let xtile = (lon_deg + 180.0) / 360.0 * n;
    let ytile = (1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n;
    (xtile, ytile, zoom)
}

fn center_to_top_left_coordinates(x_pixels: u32, y_pixels: u32) -> (u32, u32) {
    // the center coordniates are usefull for orienting ourselves in one tile,
    // but for drawing them, top left is better
    let y_to_img_border = 512 + y_pixels;
    let y_img_coords = y_to_img_border - (630 - 125) / 2;
    let x_to_img_border = 512 * 2 + x_pixels;
    let x_img_coords = x_to_img_border - 1200 / 2;
    (x_img_coords, y_img_coords)
}

fn is_in_range(x_pixels: u32, y_pixels: u32, x_index: u32, y_index: u32) -> bool {
    let x_in_range = x_pixels <= (x_index + 1) * 512 && x_pixels + 1200 >= x_index * 512;
    let y_in_range = y_pixels <= (y_index + 1) * 512 && y_pixels + (630 - 125) >= y_index * 512;
    x_in_range && y_in_range
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_lat_lon_z_to_xyz() {
        let (x, y, _) = lat_lon_z_to_xyz(52.520_008, 13.404_954, 17);
        assert_eq!(x, 70416.59480746667_f64);
        assert_eq!(y, 42985.734050611834_f64);
    }

    #[test]
    fn test_lat_lon_no_zoom_mut() {
        for x in -5..5 {
            let x = x as f64;
            for y in -5..5 {
                let y = y as f64;
                for z in 0..20 {
                    let (_, _, zg) = lat_lon_z_to_xyz(x + y / 100.0, y, z);
                    assert_eq!(z, zg);
                    let (_, _, zg) = lat_lon_z_to_xyz(x, y + x / 100.0, z);
                    assert_eq!(z, zg);
                }
            }
        }
    }

    fn assert_range_eq(
        x_pixels: u32,
        y_pixels: u32,
        expected_x: (u32, u32),
        expected_y: (u32, u32),
    ) {
        for x in 0..10 {
            for y in 0..10 {
                let (x_min, x_max) = expected_x;
                let (y_min, y_max) = expected_y;
                let expected_result = x <= x_max && x >= x_min && y <= y_max && y >= y_min;
                assert_eq!(is_in_range(x_pixels, y_pixels, x, y), expected_result);
            }
        }
    }

    #[test]
    fn ranged_test() {
        assert_range_eq(0, 0, (0, 2), (0, 0));
        assert_range_eq(0, 513, (0, 2), (1, 1));
        assert_range_eq(512 / 2, 0, (0, 2), (0, 0));
        assert_range_eq(512 / 2, 512 / 2, (0, 2), (0, 1));
    }
}
