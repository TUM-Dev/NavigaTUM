use futures::{stream::FuturesUnordered, StreamExt};
use log::warn;
use std::ops::Range;

use crate::maps::fetch_tile::FetchTileTask;
use crate::models::Location;

pub struct OverlayMapTask {
    pub x: f64,
    pub y: f64,
    pub z: u32,
}

const POSSIBLE_INDEX_RANGE: Range<u32> = 0..7;

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
        // around this center there is a 5*5 grid of tiles
        // -------------------------------
        // | -1 /  1 |  0 /  1 |  1 /  1 |
        // | -1 /  0 |    x    |  1 /  0 |
        // | -1 / -1 |  0 / -1 |  1 / -1 |
        // -------------------------------
        // we can now filter for "is on the image" and append them to a work queue

        let x_pixels = (512.0 * (self.x - self.x.floor())) as u32;
        let y_pixels = (512.0 * (self.y - self.y.floor())) as u32;
        let (x_img_coords, y_img_coords) = center_to_top_left_coordinates(img, x_pixels, y_pixels);
        // is_in_range is quite cheap => we over-check this one to cope with different image formats
        let mut work_queue = FuturesUnordered::new();
        for index_x in POSSIBLE_INDEX_RANGE.clone() {
            for index_y in POSSIBLE_INDEX_RANGE.clone() {
                if is_on_image(img, (x_img_coords, y_img_coords), (index_x, index_y)) {
                    let offset_x = (index_x as i32) - ((POSSIBLE_INDEX_RANGE.end / 2) as i32);
                    let offset_y = (index_y as i32) - ((POSSIBLE_INDEX_RANGE.end / 2) as i32);
                    work_queue.push(
                        FetchTileTask::from(self)
                            .offset_by(offset_x, offset_y)
                            .with_index(index_x, index_y)
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

/// The center coordinates are usefully for orienting ourselves in one tile
/// For drawing them, top left is better
fn center_to_top_left_coordinates(
    img: &image::RgbaImage,
    x_pixels: u32,
    y_pixels: u32,
) -> (u32, u32) {
    let y_to_img_border = 512 * (POSSIBLE_INDEX_RANGE.end / 2) + y_pixels;
    let y_img_coords = y_to_img_border - (img.height() - 125) / 2;
    let x_to_img_border = 512 * (POSSIBLE_INDEX_RANGE.end / 2) + x_pixels;
    let x_img_coords = x_to_img_border - img.width() / 2;
    (x_img_coords, y_img_coords)
}

fn is_on_image(
    img: &image::RgbaImage,
    (x_pixel, y_pixel): (u32, u32),
    (x_index, y_index): (u32, u32),
) -> bool {
    let x_in_range = (x_index + 1) * 512 >= x_pixel && x_index * 512 <= x_pixel + img.width();
    let y_in_range =
        (y_index + 1) * 512 >= y_pixel && y_index * 512 <= y_pixel + (img.height() - 125);
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
        let img = image::RgbaImage::new(1200, 630);
        for x in 0..10 {
            for y in 0..10 {
                let (x_min, x_max) = expected_x;
                let (y_min, y_max) = expected_y;
                let expected_result = x <= x_max && x >= x_min && y <= y_max && y >= y_min;
                assert_eq!(
                    is_on_image(&img, (x_pixels, y_pixels), (x, y)),
                    expected_result
                );
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
