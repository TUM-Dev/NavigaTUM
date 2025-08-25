use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::AppliableEdit;

#[derive(Deserialize, Debug, Clone, Copy, Default, PartialEq, utoipa::ToSchema)]
pub struct Coordinate {
    /// Latitude
    #[schema(example = 48.26244490906312)]
    lat: f64,
    /// Longitude
    #[schema(example = 48.26244490906312)]
    lon: f64,
}

impl Coordinate {
    fn get_coordinates_csv_path(base_dir: &Path) -> PathBuf {
        base_dir
            .join("data")
            .join("sources")
            .join("coordinates.csv")
    }
}
impl AppliableEdit for Coordinate {
    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> String {
        use std::io::{BufRead, BufReader, BufWriter, Write};

        let csv_file = Self::get_coordinates_csv_path(base_dir);
        let temp_file = csv_file.with_extension("tmp");

        {
            // Write header
            let output = std::fs::File::create(&temp_file).unwrap();
            let mut writer = BufWriter::new(output);
            writeln!(writer, "id,lat,lon").unwrap();

            let mut wrote_edit = false;

            // Process remaining lines
            {
                let input = std::fs::File::open(&csv_file).unwrap();
                for line in BufReader::new(input)
                    .lines()
                    .skip(1)
                    .map_while(Result::ok)
                    .filter(|l| !l.trim().is_empty())
                {
                    if let Some(existing_key) = line.split(',').next() {
                        if !wrote_edit && existing_key >= key {
                            wrote_edit = true;
                            writeln!(writer, "{key},{lat},{lon}", lat = self.lat, lon = self.lon)
                                .unwrap();
                        }
                        if existing_key != key {
                            writeln!(writer, "{line}").unwrap();
                        }
                    }
                }
            }

            // Append at the end
            if !wrote_edit {
                writeln!(writer, "{key},{lat},{lon}", lat = self.lat, lon = self.lon).unwrap();
            }
        }

        // Replace original file with temp file
        std::fs::rename(&temp_file, &csv_file).unwrap();

        format!(
            "https://nav.tum.de/api/preview_edit/{key}?to_lat={lat}&to_lon={lon}",
            lat = self.lat,
            lon = self.lon
        )
    }
}

#[cfg(test)]
mod tests {
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
        coord.apply("2", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n2,0,0\n"
        );
        coord.apply("000.991", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n000.991,0,0\n2,0,0\n"
        );
        coord.apply("1", dir.path(), "branch");
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
        coord.apply("1", dir.path(), "branch");
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
        coord.apply("beta", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\nalpha,1.0,1.0\nbeta,0,0\nzulu,1.0,1.0\n"
        );

        // inserting numbers
        coord.apply("0", dir.path(), "branch");
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
        coord.apply("0", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,1.0,1.0\n"
        );
        coord.apply("1", dir.path(), "branch");
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
        coord.apply("0", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,1.0,1.0\n"
        );
        coord.apply("1", dir.path(), "branch");
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

        coord.apply("1", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,1.0,1.0\n1,0,0\n2,1.0,1.0\n"
        );

        coord.apply("0", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,0,0\n2,1.0,1.0\n"
        );

        coord.apply("2", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n1,0,0\n2,0,0\n"
        );
    }

    #[test]
    fn test_empty_csv_insertion() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();

        coord.apply("1", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n1,0,0\n"
        );
    }

    #[test]
    fn test_newline_at_eof() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        coord.apply("0", dir.path(), "branch");
        let expected = "id,lat,lon\n0,0,0\n";
        assert_eq!(fs::read_to_string(&csv_file).unwrap(), expected);
    }
}
