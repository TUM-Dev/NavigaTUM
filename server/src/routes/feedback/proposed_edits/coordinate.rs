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
        let csv_file = Self::get_coordinates_csv_path(base_dir);
        let content = std::fs::read_to_string(&csv_file).unwrap();
        let mut lines = content.lines().collect::<Vec<&str>>();

        // Find existing entry
        let pos_of_line_to_edit = lines
            .iter()
            .skip(1) // Skip header
            .position(|l| l.starts_with(&format!("{key},")));

        let new_line = format!("{},{},{}", key, self.lat, self.lon);

        if let Some(pos) = pos_of_line_to_edit {
            // Update existing entry (add 1 to account for header)
            lines[pos + 1] = &new_line;
        } else {
            // Insert new entry in sorted position
            let mut insert_pos = 1; // Start after header
            for (i, line) in lines.iter().skip(1).enumerate() {
                if let Some(existing_key) = line.split(',').next() {
                    if existing_key > key {
                        insert_pos = i;
                        break;
                    }
                    insert_pos = i;
                }
            }
            lines.insert(insert_pos, &new_line);
        }

        let content = lines.join("\n");
        let content = format!("{}\n", content.trim_end());
        std::fs::write(&csv_file, content).unwrap();

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
    fn test_edit_incorrectly_sorted() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        fs::write(&csv_file, "id,lat,lon\n1,1.0,1.0\n0,1.0,1.0\n").unwrap();
        coord.apply("0", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n1,1.0,1.0\n0,0,0\n"
        );
        coord.apply("1", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n1,0,0\n0,0,0\n"
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

        coord.apply("0", dir.path(), "branch");
        assert_eq!(
            fs::read_to_string(&csv_file).unwrap(),
            "id,lat,lon\n0,0,0\n"
        );
    }

    #[test]
    fn test_newline_at_eof() {
        let coord = Coordinate::default();
        let (dir, csv_file) = setup();
        coord.apply("0", dir.path(), "branch");
        let expected = "id,lat,lon\n0,0,0\n";
        assert_eq!(fs::read_to_string(&csv_file).unwrap(), expected);

        // Test that multiple newlines at EOF are handled correctly
        fs::write(&csv_file, "id,lat,lon\n0,0,0\n\n\n").unwrap();
        coord.apply("0", dir.path(), "branch");
        // The content should be normalized to have exactly one newline at EOF
        let result = fs::read_to_string(&csv_file).unwrap();
        assert!(result.ends_with("0,0,0\n"));
        assert!(!result.ends_with("0,0,0\n\n"));
    }
}
