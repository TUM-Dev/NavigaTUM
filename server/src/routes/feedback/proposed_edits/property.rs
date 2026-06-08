use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::AppliableEdit;
use super::csv_edit::apply_csv_upsert;

#[derive(Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
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

impl PropertyEdit {
    fn names_csv_path(base_dir: &Path) -> PathBuf {
        base_dir.join("data").join("sources").join("names.csv")
    }
    fn usages_csv_path(base_dir: &Path) -> PathBuf {
        base_dir.join("data").join("sources").join("usages.csv")
    }
}

impl AppliableEdit for PropertyEdit {
    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
        match self {
            Self::Name { name, short_name } => {
                let name_val = name.as_deref().unwrap_or("");
                let short_val = short_name.as_deref().unwrap_or("");
                apply_csv_upsert(
                    key,
                    &Self::names_csv_path(base_dir),
                    &[key.to_string(), name_val.to_string(), short_val.to_string()],
                )?;
                Ok(format!("name: `{name_val}`, short_name: `{short_val}`"))
            }
            Self::Usage {
                name_de,
                name_en,
                din_277,
                din_277_desc,
            } => {
                let din = din_277.as_deref().unwrap_or("");
                let din_desc = din_277_desc.as_deref().unwrap_or("");
                apply_csv_upsert(
                    key,
                    &Self::usages_csv_path(base_dir),
                    &[
                        key.to_string(),
                        name_de.clone(),
                        name_en.clone(),
                        din.to_string(),
                        din_desc.to_string(),
                    ],
                )?;
                Ok(format!(
                    "usage: `{name_de}` / `{name_en}` (DIN 277: `{din}`)"
                ))
            }
            Self::Link {
                text_de,
                text_en,
                url,
            } => {
                let yaml_path = base_dir.join("data").join("sources").join("links.yaml");

                let mut links: BTreeMap<String, Vec<LinkEntry>> = if yaml_path.exists() {
                    let file = File::open(&yaml_path)?;
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

                let file = File::create(&yaml_path)?;
                serde_yaml::to_writer(file, &links)?;

                Ok(format!("link: [`{text_de}` / `{text_en}`]({url})"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        reason = "tests assert via panic/unwrap"
    )]
    use std::fs;

    use insta::assert_snapshot;

    use super::*;

    const NAMES_HEADER: &str = "id,name,short_name,arch_name";
    const USAGES_HEADER: &str = "id,name_de,name_en,din_277,din_277_desc,din277_name";

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().unwrap();
        let sources_dir = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources_dir).unwrap();

        fs::write(sources_dir.join("names.csv"), format!("{NAMES_HEADER}\n")).unwrap();
        fs::write(sources_dir.join("usages.csv"), format!("{USAGES_HEADER}\n")).unwrap();

        (dir, sources_dir)
    }

    #[test]
    fn test_name_edit_insert() {
        let (dir, sources_dir) = setup();
        let edit = PropertyEdit::Name {
            name: Some("Test Room".to_string()),
            short_name: Some("TR".to_string()),
        };
        let desc = edit.apply("0101.01.001", dir.path(), "branch").unwrap();
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
            format!("{NAMES_HEADER}\nalpha,A,,\nzulu,Z,,\n"),
        )
        .unwrap();

        let edit = PropertyEdit::Name {
            name: Some("Beta".to_string()),
            short_name: None,
        };
        edit.apply("beta", dir.path(), "branch").unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r"
        id,name,short_name,arch_name
        alpha,A,,
        beta,Beta,,
        zulu,Z,,
        ");
    }

    #[test]
    fn test_name_edit_preserves_arch_name() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("names.csv"),
            format!("{NAMES_HEADER}\nalpha,Old,SN,1951\n"),
        )
        .unwrap();

        let edit = PropertyEdit::Name {
            name: Some("New".to_string()),
            short_name: Some("N".to_string()),
        };
        edit.apply("alpha", dir.path(), "branch").unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r"
        id,name,short_name,arch_name
        alpha,New,N,1951
        ");
    }

    #[test]
    fn test_usage_edit_insert_pads_to_full_width() {
        let (dir, sources_dir) = setup();
        let edit = PropertyEdit::Usage {
            name_de: "Büro".to_string(),
            name_en: "Office".to_string(),
            din_277: Some("NF2.1".to_string()),
            din_277_desc: Some("Büroräume".to_string()),
        };
        let desc = edit.apply("room1", dir.path(), "branch").unwrap();
        assert_snapshot!(desc, @"usage: `Büro` / `Office` (DIN 277: `NF2.1`)");
        assert_snapshot!(fs::read_to_string(sources_dir.join("usages.csv")).unwrap(), @r"
        id,name_de,name_en,din_277,din_277_desc,din277_name
        room1,Büro,Office,NF2.1,Büroräume,
        ");
    }

    #[test]
    fn test_usage_edit_preserves_din277_name() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("usages.csv"),
            format!("{USAGES_HEADER}\nroom1,Werkstatt,Workshop,NF3.2,,Werkstätten\n"),
        )
        .unwrap();
        let edit = PropertyEdit::Usage {
            name_de: "Labor".to_string(),
            name_en: "Lab".to_string(),
            din_277: Some("NF3.3".to_string()),
            din_277_desc: Some("Labor".to_string()),
        };
        edit.apply("room1", dir.path(), "branch").unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("usages.csv")).unwrap(), @r"
        id,name_de,name_en,din_277,din_277_desc,din277_name
        room1,Labor,Lab,NF3.3,Labor,Werkstätten
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
        let desc = edit.apply("room1", dir.path(), "branch").unwrap();
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
        edit.apply("room1", dir.path(), "branch").unwrap();
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
        edit.apply("test", dir.path(), "branch").unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r#"
        id,name,short_name,arch_name
        test,"Room, with comma",,
        "#);
    }
}
