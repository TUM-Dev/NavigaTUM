use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::AppliableEdit;

#[derive(Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(tag = "type")]
pub enum PropertyEdit {
    Name {
        name: Option<String>,
        short_name: Option<String>,
    },
    Usage {
        name_de: String,
        name_en: String,
        din_277: Option<String>,
        din_277_desc: Option<String>,
    },
    Link {
        text_de: String,
        text_en: String,
        url: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LinkEntry {
    text: BTreeMap<String, String>,
    url: String,
}

// TODO: AppliableEdit::apply returns String and cannot propagate I/O errors;
// the existing helpers panic on filesystem failure. Refactor to Result before
// removing the allow.
#[allow(clippy::unwrap_used, clippy::panic)]
impl PropertyEdit {
    fn names_csv_path(base_dir: &Path) -> PathBuf {
        base_dir.join("data").join("sources").join("names.csv")
    }
    fn usages_csv_path(base_dir: &Path) -> PathBuf {
        base_dir.join("data").join("sources").join("usages.csv")
    }

    fn apply_csv_edit(
        key: &str,
        base_dir: &Path,
        csv_path_fn: fn(&Path) -> PathBuf,
        header: &str,
        make_line: impl FnOnce() -> String,
    ) {
        use std::io::{BufRead as _, BufReader, BufWriter, Write as _};

        let csv_file = csv_path_fn(base_dir);
        let temp_file = csv_file.with_extension("tmp");

        {
            let output = File::create(&temp_file).unwrap();
            let mut writer = BufWriter::new(output);
            writeln!(writer, "{header}").unwrap();

            let mut wrote_edit = false;
            let new_line = make_line();

            {
                let input = File::open(&csv_file).unwrap();
                for line in BufReader::new(input)
                    .lines()
                    .skip(1)
                    .map_while(Result::ok)
                    .filter(|l| !l.trim().is_empty())
                {
                    if let Some(existing_key) = line.split(',').next() {
                        if !wrote_edit && existing_key >= key {
                            wrote_edit = true;
                            writeln!(writer, "{new_line}").unwrap();
                        }
                        if existing_key != key {
                            writeln!(writer, "{line}").unwrap();
                        }
                    }
                }
            }

            if !wrote_edit {
                writeln!(writer, "{new_line}").unwrap();
            }
        }

        fs::rename(&temp_file, &csv_file).unwrap();
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[allow(clippy::unwrap_used, clippy::panic)]
impl AppliableEdit for PropertyEdit {
    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> String {
        match self {
            Self::Name { name, short_name } => {
                let name_val = name.as_deref().unwrap_or("");
                let short_val = short_name.as_deref().unwrap_or("");
                Self::apply_csv_edit(
                    key,
                    base_dir,
                    Self::names_csv_path,
                    "id,name,short_name,arch_name",
                    || {
                        format!(
                            "{},{},{},",
                            csv_escape(key),
                            csv_escape(name_val),
                            csv_escape(short_val)
                        )
                    },
                );
                format!("name: `{name_val}`, short_name: `{short_val}`")
            }
            Self::Usage {
                name_de,
                name_en,
                din_277,
                din_277_desc,
            } => {
                let din = din_277.as_deref().unwrap_or("");
                let din_desc = din_277_desc.as_deref().unwrap_or("");
                Self::apply_csv_edit(
                    key,
                    base_dir,
                    Self::usages_csv_path,
                    "id,name_de,name_en,din_277,din_277_desc",
                    || {
                        format!(
                            "{},{},{},{},{}",
                            csv_escape(key),
                            csv_escape(name_de),
                            csv_escape(name_en),
                            csv_escape(din),
                            csv_escape(din_desc),
                        )
                    },
                );
                format!("usage: `{name_de}` / `{name_en}` (DIN 277: `{din}`)")
            }
            Self::Link {
                text_de,
                text_en,
                url,
            } => {
                let yaml_path = base_dir.join("data").join("sources").join("links.yaml");

                let mut links: BTreeMap<String, Vec<LinkEntry>> = if yaml_path.exists() {
                    let file = File::open(&yaml_path).unwrap();
                    serde_yaml::from_reader(file).unwrap_or_default()
                } else {
                    BTreeMap::new()
                };

                let entry = LinkEntry {
                    text: BTreeMap::from([
                        ("de".to_string(), text_de.clone()),
                        ("en".to_string(), text_en.clone()),
                    ]),
                    url: url.clone(),
                };

                links.entry(key.to_string()).or_default().push(entry);

                let file = File::create(&yaml_path).unwrap();
                serde_yaml::to_writer(file, &links).unwrap();

                format!("link: [`{text_de}` / `{text_en}`]({url})")
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
    use std::fs;

    use insta::assert_snapshot;

    use super::*;

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().unwrap();
        let sources_dir = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources_dir).unwrap();

        fs::write(
            sources_dir.join("names.csv"),
            "id,name,short_name,arch_name\n",
        )
        .unwrap();
        fs::write(
            sources_dir.join("usages.csv"),
            "id,name_de,name_en,din_277,din_277_desc\n",
        )
        .unwrap();

        (dir, sources_dir)
    }

    #[test]
    fn test_name_edit_insert() {
        let (dir, sources_dir) = setup();
        let edit = PropertyEdit::Name {
            name: Some("Test Room".to_string()),
            short_name: Some("TR".to_string()),
        };
        let desc = edit.apply("0101.01.001", dir.path(), "branch");
        assert_snapshot!(desc, @r#"name: `Test Room`, short_name: `TR`"#);
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r"
        id,name,short_name,arch_name
        0101.01.001,Test Room,TR,
        ");
    }

    #[test]
    fn test_name_edit_sorted_insert() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("names.csv"),
            "id,name,short_name,arch_name\nalpha,A,,\nzulu,Z,,\n",
        )
        .unwrap();

        let edit = PropertyEdit::Name {
            name: Some("Beta".to_string()),
            short_name: None,
        };
        edit.apply("beta", dir.path(), "branch");
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r"
        id,name,short_name,arch_name
        alpha,A,,
        beta,Beta,,
        zulu,Z,,
        ");
    }

    #[test]
    fn test_name_edit_update() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("names.csv"),
            "id,name,short_name,arch_name\nalpha,Old,,\n",
        )
        .unwrap();

        let edit = PropertyEdit::Name {
            name: Some("New".to_string()),
            short_name: Some("N".to_string()),
        };
        edit.apply("alpha", dir.path(), "branch");
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r"
        id,name,short_name,arch_name
        alpha,New,N,
        ");
    }

    #[test]
    fn test_usage_edit() {
        let (dir, sources_dir) = setup();
        let edit = PropertyEdit::Usage {
            name_de: "Büro".to_string(),
            name_en: "Office".to_string(),
            din_277: Some("NF2.1".to_string()),
            din_277_desc: Some("Büroräume".to_string()),
        };
        let desc = edit.apply("room1", dir.path(), "branch");
        assert_snapshot!(desc, @"usage: `Büro` / `Office` (DIN 277: `NF2.1`)");
        assert_snapshot!(fs::read_to_string(sources_dir.join("usages.csv")).unwrap(), @r"
        id,name_de,name_en,din_277,din_277_desc
        room1,Büro,Office,NF2.1,Büroräume
        ");
    }

    #[test]
    fn test_link_edit() {
        let (dir, sources_dir) = setup();
        let edit = PropertyEdit::Link {
            text_de: "Homepage".to_string(),
            text_en: "Homepage".to_string(),
            url: "https://example.com".to_string(),
        };
        let desc = edit.apply("room1", dir.path(), "branch");
        assert_snapshot!(desc, @"link: [`Homepage` / `Homepage`](https://example.com)");
        assert_snapshot!(fs::read_to_string(sources_dir.join("links.yaml")).unwrap(), @r"
        room1:
        - text:
            de: Homepage
            en: Homepage
          url: https://example.com
        ");
    }

    #[test]
    fn test_link_edit_appends() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("links.yaml"),
            "room1:\n- text:\n    de: Existing\n    en: Existing\n  url: https://old.com\n",
        )
        .unwrap();

        let edit = PropertyEdit::Link {
            text_de: "New".to_string(),
            text_en: "New".to_string(),
            url: "https://new.com".to_string(),
        };
        edit.apply("room1", dir.path(), "branch");
        assert_snapshot!(fs::read_to_string(sources_dir.join("links.yaml")).unwrap(), @r"
        room1:
        - text:
            de: Existing
            en: Existing
          url: https://old.com
        - text:
            de: New
            en: New
          url: https://new.com
        ");
    }

    #[test]
    fn test_csv_escape_commas() {
        let (dir, sources_dir) = setup();
        let edit = PropertyEdit::Name {
            name: Some("Room, with comma".to_string()),
            short_name: None,
        };
        edit.apply("test", dir.path(), "branch");
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r#"
        id,name,short_name,arch_name
        test,"Room, with comma",,
        "#);
    }
}
