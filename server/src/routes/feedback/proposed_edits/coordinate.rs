use std::fs::{self, File};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::AppliableEdit;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq, utoipa::ToSchema)]
pub struct Coordinate {
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
}

impl Coordinate {
    pub(super) fn lat(&self) -> f64 {
        self.lat
    }
    pub(super) fn lon(&self) -> f64 {
        self.lon
    }

    fn get_coordinates_csv_path(base_dir: &Path) -> PathBuf {
        base_dir
            .join("data")
            .join("sources")
            .join("coordinates.csv")
    }

    /// Centralised so callers can't accidentally swap `[lon, lat]` (RFC 7946 ordering).
    pub(super) fn fenced_geojson_feature(&self, properties: &serde_json::Value) -> String {
        let geojson = serde_json::json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [self.lon, self.lat]
            },
            "properties": properties,
        });
        let pretty = serde_json::to_string_pretty(&geojson).unwrap_or_else(|_| geojson.to_string());
        format!("```geojson\n{pretty}\n```")
    }

    pub(super) fn apply_to_csv(&self, key: &str, base_dir: &Path) -> anyhow::Result<()> {
        use std::io::{BufRead as _, BufReader, BufWriter, Write as _};

        let csv_file = Self::get_coordinates_csv_path(base_dir);
        let temp_file = csv_file.with_extension("tmp");

        {
            let output = File::create(&temp_file)?;
            let mut writer = BufWriter::new(output);
            writeln!(writer, "id,lat,lon")?;

            let mut wrote_edit = false;

            {
                let input = File::open(&csv_file)?;
                for line in BufReader::new(input)
                    .lines()
                    .skip(1)
                    .map_while(Result::ok)
                    .filter(|l| !l.trim().is_empty())
                {
                    if let Some(existing_key) = line.split(',').next() {
                        if !wrote_edit && existing_key >= key {
                            wrote_edit = true;
                            writeln!(writer, "{key},{lat},{lon}", lat = self.lat, lon = self.lon)?;
                        }
                        if existing_key != key {
                            writeln!(writer, "{line}")?;
                        }
                    }
                }
            }

            if !wrote_edit {
                writeln!(writer, "{key},{lat},{lon}", lat = self.lat, lon = self.lon)?;
            }
        }

        fs::rename(&temp_file, &csv_file)?;
        Ok(())
    }
}

impl AppliableEdit for Coordinate {
    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
        self.apply_to_csv(key, base_dir)?;
        Ok(self.fenced_geojson_feature(&serde_json::json!({
            "kind": "coordinate-change",
            "id": key,
            "to_lat": self.lat,
            "to_lon": self.lon,
        })))
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::indexing_slicing,
        reason = "tests assert via panic/unwrap and index into known-shape JSON fixtures"
    )]
    use std::fs;

    use pretty_assertions::assert_eq;

    use super::*;

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().unwrap();
        let sources_dir = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources_dir).unwrap();

        let csv_file = sources_dir.join("coordinates.csv");
        fs::write(&csv_file, "id,lat,lon\n").unwrap();
        (dir, csv_file)
    }

    #[test]
    fn test_get_coordinates_csv_path() {
        let (dir, csv_file) = setup();
        assert_eq!(Coordinate::get_coordinates_csv_path(dir.path()), csv_file);
    }

    #[test]
    fn test_insertion_alphabetical() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        fs::write(&csv_file, "id,lat,lon\n0,1.0,1.0\n").unwrap();
        coord.apply("2", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n2,0,0\n"
        );
        coord.apply("000.991", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n000.991,0,0\n2,0,0\n"
        );
        coord.apply("1", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n000.991,0,0\n1,0,0\n2,0,0\n"
        );
    }

    #[test]
    fn test_prefix_alphabetical() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        fs::write(&csv_file, "id,lat,lon\n0,1.0,1.0\n2,1.0,1.0\n").unwrap();
        coord.apply("1", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n1,0,0\n2,1.0,1.0\n"
        );
    }

    #[test]
    fn test_insertion_char_friendly() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        fs::write(&csv_file, "id,lat,lon\nalpha,1.0,1.0\nzulu,1.0,1.0\n").unwrap();
        // inserting chars works
        coord.apply("beta", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\nalpha,1.0,1.0\nbeta,0,0\nzulu,1.0,1.0\n"
        );

        // inserting numbers
        coord.apply("0", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\nalpha,1.0,1.0\nbeta,0,0\nzulu,1.0,1.0\n"
        );
    }

    #[test]
    fn test_edit_correctly_sorted() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        fs::write(&csv_file, "id,lat,lon\n0,1.0,1.0\n1,1.0,1.0\n").unwrap();
        coord.apply("0", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,1.0,1.0\n"
        );
        coord.apply("1", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,0,0\n"
        );
    }

    #[test]
    fn test_edit_correctly_sorted_updates() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        fs::write(&csv_file, "id,lat,lon\n0,1.0,1.0\n1,1.0,1.0\n").unwrap();
        coord.apply("0", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,1.0,1.0\n"
        );
        coord.apply("1", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,0,0\n"
        );
    }

    #[test]
    fn test_basic_csv_operations() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        fs::write(&csv_file, "id,lat,lon\n0,1.0,1.0\n2,1.0,1.0\n").unwrap();

        coord.apply("1", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n1,0,0\n2,1.0,1.0\n"
        );

        coord.apply("0", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,0,0\n2,1.0,1.0\n"
        );

        coord.apply("2", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,0,0\n2,0,0\n"
        );
    }

    #[test]
    fn test_empty_csv_insertion() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();

        coord.apply("1", dir.path(), "branch").unwrap();
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n1,0,0\n"
        );
    }

    #[test]
    fn test_newline_at_eof() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        coord.apply("0", dir.path(), "branch").unwrap();
        let expected = "id,lat,lon\n0,0,0\n";
        assert_eq!(fs::read_to_string(&csv_file).unwrap(), expected);
    }

    #[test]
    fn test_apply_returns_geojson_block() {
        let coord = Coordinate {
            lat: 48.262,
            lon: 11.668,
        };
        let (dir, _csv_file) = setup();
        let result = coord.apply("mi", dir.path(), "branch").unwrap();

        // The result must start and end with the geojson fenced block markers.
        assert!(
            result.starts_with("```geojson\n"),
            "expected fenced geojson block, got: {result}"
        );
        assert!(
            result.ends_with("\n```"),
            "expected closing fence, got: {result}"
        );

        // Strip the fences and parse as JSON.
        let json_str = result
            .strip_prefix("```geojson\n")
            .unwrap()
            .strip_suffix("\n```")
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(json_str)
            .expect("apply() must return valid JSON inside the fenced block");

        assert_eq!(value["type"], "Feature");
        assert_eq!(value["geometry"]["type"], "Point");
        // GeoJSON coordinate order is [longitude, latitude].
        assert_eq!(value["geometry"]["coordinates"][0], 11.668);
        assert_eq!(value["geometry"]["coordinates"][1], 48.262);
        assert_eq!(value["properties"]["kind"], "coordinate-change");
        assert_eq!(value["properties"]["id"], "mi");
        assert_eq!(value["properties"]["to_lat"], 48.262);
        assert_eq!(value["properties"]["to_lon"], 11.668);
    }
}
