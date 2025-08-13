use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::AppliableEdit;

struct CoordinateFile {
    path: PathBuf,
}

impl From<PathBuf> for CoordinateFile {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl CoordinateFile {
    fn matches(&self) -> Range<u32> {
        let name = self.path.file_name().unwrap().to_str().unwrap();
        let prefix = name
            .split('_')
            .next()
            .unwrap_or(name)
            .split('.')
            .next()
            .unwrap();
        let range = prefix
            .split('-')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect::<Vec<u32>>();
        match range.len() {
            0 => 0..99999,
            1 => range[0]..range[0],
            2 => range[0]..range[1],
            _ => panic!("Invalid range: {range:?}"),
        }
    }
}

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
    fn best_matching_file(key: &str, base_dir: &Path) -> PathBuf {
        // we extract the building from the key, defaulting to 90000, as this is out of the range of all buildings
        let key = key.split('-').next().unwrap_or(key);
        let building = key.parse::<u32>().unwrap_or(90000);

        let coord_dir = base_dir.join("data").join("sources").join("coordinates");
        let filenames = std::fs::read_dir(coord_dir)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .filter_map(Result::ok)
            .map(CoordinateFile::from);
        let best_match = filenames
            .filter(|co| co.matches().contains(&building))
            .min_by_key(|f| {
                let Range { start, end } = f.matches();
                end - start
            })
            .expect("No matching file found, which is impossible");
        best_match.path
    }
}
impl AppliableEdit for Coordinate {
    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> String {
        let file = Self::best_matching_file(key, base_dir);
        let content = std::fs::read_to_string(file.clone()).unwrap();
        let mut lines = content.lines().collect::<Vec<&str>>();
        let pos_of_line_to_edit = lines
            .iter()
            .position(|l| l.starts_with(&format!("\"{key}\": ")));
        let mut new_line = format!(
            "\"{key}\": {{ lat: {lat}, lon: {lon} }}",
            lat = self.lat,
            lon = self.lon,
        );

        if let Some(pos) = pos_of_line_to_edit {
            // persist comments
            if lines[pos].contains('#') {
                new_line += " #";
                new_line += lines[pos].split('#').next_back().unwrap();
            }
            lines[pos] = &new_line;
        } else {
            //we need to insert a new line at a fitting position
            let pos_of_line_to_insert = lines
                .iter()
                .position(|l| {
                    let key_at_pos = l.split("\":").next().unwrap().strip_prefix('"');
                    key_at_pos > Some(key)
                })
                .unwrap_or(lines.len());
            lines.insert(pos_of_line_to_insert, &new_line);
        }
        let content = lines.join("\n").trim().to_string();
        std::fs::write(file.as_path(), content + "\n").unwrap();
        format!(
            "https://nav.tum.de/api/preview_edit/{k}?to_lat={lat}&to_lon={lon}",
            k = key,
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
        let coord_dir = dir.path().join("data").join("sources").join("coordinates");
        fs::create_dir_all(&coord_dir).unwrap();

        fs::write(coord_dir.join("100-200.yaml"), "").unwrap();
        fs::write(coord_dir.join("100-110.yaml"), "").unwrap();
        fs::write(coord_dir.join("101-102.yaml"), "").unwrap();
        fs::write(coord_dir.join("rest.yaml"), "").unwrap();
        (dir, coord_dir.join("rest.yaml"))
    }

    #[test]
    fn test_matches() {
        let file = CoordinateFile::from(PathBuf::from("data/sources/coordinate/0-100.json"));
        std::assert_eq!(file.matches(), 0..100);
        let file = CoordinateFile::from(PathBuf::from("data/sources/coordinate/42.json"));
        std::assert_eq!(file.matches(), 42..42);
        let file = CoordinateFile::from(PathBuf::from("data/sources/coordinate/0-100_abc.json"));
        std::assert_eq!(file.matches(), 0..100);
        let file = CoordinateFile::from(PathBuf::from("data/sources/coordinate/42_abc.json"));
        std::assert_eq!(file.matches(), 42..42);
        let file = CoordinateFile::from(PathBuf::from("data/sources/coordinate/rest.yaml"));
        std::assert_eq!(file.matches(), 0..99999);
    }

    #[test]
    fn test_best_matching_file() {
        let (dir, target_file) = setup();
        assert_eq!(
            Coordinate::best_matching_file("100", dir.path()),
            target_file.parent().unwrap().join("100-110.yaml")
        );
        assert_eq!(
            Coordinate::best_matching_file("101", dir.path()),
            target_file.parent().unwrap().join("101-102.yaml")
        );
        assert_eq!(
            Coordinate::best_matching_file("130", dir.path()),
            target_file.parent().unwrap().join("100-200.yaml")
        );
        assert_eq!(
            Coordinate::best_matching_file("11", dir.path()),
            target_file
        );
        assert_eq!(
            Coordinate::best_matching_file("300", dir.path()),
            target_file
        );
        assert_eq!(
            Coordinate::best_matching_file("mi", dir.path()),
            target_file
        );
    }

    #[test]
    fn test_insertion_alphabetical() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        fs::write(&target_file, "\"0\": { lat: 1.0, lon: 1.0 }\n").unwrap();
        coord.apply("2", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 1.0, lon: 1.0 }\n\"2\": { lat: 0, lon: 0 }\n"
        );
        coord.apply("000.991", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 1.0, lon: 1.0 }\n\"000.991\": { lat: 0, lon: 0 }\n\"2\": { lat: 0, lon: 0 }\n"
        );
        coord.apply("1", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 1.0, lon: 1.0 }\n\"000.991\": { lat: 0, lon: 0 }\n\"1\": { lat: 0, lon: 0 }\n\"2\": { lat: 0, lon: 0 }\n"
        );
    }

    #[test]
    fn test_prefix_alphabetical() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        fs::write(
            &target_file,
            "\"0\": { lat: 1.0, lon: 1.0 }\n\"2\": { lat: 1.0, lon: 1.0 }\n",
        )
        .unwrap();
        coord.apply("1", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 1.0, lon: 1.0 }\n\"1\": { lat: 0, lon: 0 }\n\"2\": { lat: 1.0, lon: 1.0 }\n"
        );
    }

    #[test]
    fn test_insertion_char_friendly() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        fs::write(
            &target_file,
            "\"alpha\": { lat: 1.0, lon: 1.0 }\n\"zulu\": { lat: 1.0, lon: 1.0 }\n",
        )
        .unwrap();
        // inserting chars works
        coord.apply("beta", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"alpha\": { lat: 1.0, lon: 1.0 }\n\"beta\": { lat: 0, lon: 0 }\n\"zulu\": { lat: 1.0, lon: 1.0 }\n"
        );

        // inserting numbers
        coord.apply("0", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 0, lon: 0 }\n\"alpha\": { lat: 1.0, lon: 1.0 }\n\"beta\": { lat: 0, lon: 0 }\n\"zulu\": { lat: 1.0, lon: 1.0 }\n"
        );
    }

    #[test]
    fn test_edit_correctly_sorted() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        fs::write(
            &target_file,
            "\"0\": { lat: 1.0, lon: 1.0 }\n\"1\": { lat: 1.0, lon: 1.0 }\n",
        )
        .unwrap();
        coord.apply("0", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 0, lon: 0 }\n\"1\": { lat: 1.0, lon: 1.0 }\n"
        );
        coord.apply("1", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 0, lon: 0 }\n\"1\": { lat: 0, lon: 0 }\n"
        );
    }

    #[test]
    fn test_edit_incorrectly_sorted() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        fs::write(
            &target_file,
            "\"1\": { lat: 1.0, lon: 1.0 }\n\"0\": { lat: 1.0, lon: 1.0 }\n",
        )
        .unwrap();
        coord.apply("0", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"1\": { lat: 1.0, lon: 1.0 }\n\"0\": { lat: 0, lon: 0 }\n"
        );
        coord.apply("1", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"1\": { lat: 0, lon: 0 }\n\"0\": { lat: 0, lon: 0 }\n"
        );
    }

    #[test]
    fn test_insertion_comment_preserving() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        fs::write(
            &target_file,
            "#comment\n\"0\": { lat: 1.0, lon: 1.0 } # inline_comment\n#comment\n\"2\": { lat: 1.0, lon: 1.0 }\n#comment\n",
        )
        .unwrap();

        coord.apply("1", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "#comment\n\"0\": { lat: 1.0, lon: 1.0 } # inline_comment\n#comment\n\"1\": { lat: 0, lon: 0 }\n\"2\": { lat: 1.0, lon: 1.0 }\n#comment\n"
        );

        coord.apply("0", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "#comment\n\"0\": { lat: 0, lon: 0 } # inline_comment\n#comment\n\"1\": { lat: 0, lon: 0 }\n\"2\": { lat: 1.0, lon: 1.0 }\n#comment\n"
        );

        coord.apply("2", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "#comment\n\"0\": { lat: 0, lon: 0 } # inline_comment\n#comment\n\"1\": { lat: 0, lon: 0 }\n\"2\": { lat: 0, lon: 0 }\n#comment\n"
        );
    }

    #[test]
    fn test_insertion_whitespace_preserving() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        fs::write(&target_file, "#abc\n\n\n#abc\n").unwrap();

        coord.apply("0", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "#abc\n\n\n#abc\n\"0\": { lat: 0, lon: 0 }\n"
        );
    }

    #[test]
    fn test_newline_at_eof() {
        let coord = Coordinate::default();
        let (dir, target_file) = setup();
        coord.apply("0", dir.path());
        let expected = "\"0\": { lat: 0, lon: 0 }\n";
        assert_eq!(fs::read_to_string(&target_file).unwrap(), expected);
        fs::write(&target_file, "\"0\": { lat: 0, lon: 0 }\n\n\n").unwrap();
        coord.apply("0", dir.path());
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "\"0\": { lat: 0, lon: 0 }\n"
        );
    }
}
