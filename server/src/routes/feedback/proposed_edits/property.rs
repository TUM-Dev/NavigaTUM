use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::Context as _;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::AppliableEdit;
use super::csv_edit::{Field, apply_csv_upsert, apply_csv_upsert_fields};

/// The data pipeline renders entries without a curated name as `{id} ({type})`
/// (e.g. `0507.01.767 (Besprechungsraum)`). A human name never starts with a
/// room/building code followed by ` (`, so this pattern reliably flags a
/// generated display name that leaked back through the edit form (#3181).
static GENERATED_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9][0-9A-Za-z@.]*\s+\(").expect("valid static regex"));

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

/// Localized values in `links.yaml` always carry exactly these two languages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct Localized {
    de: String,
    en: String,
}

/// `links.yaml` mixes plain strings and `{de, en}` maps for both fields.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Translatable {
    Localized(Localized),
    Plain(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct LinkEntry {
    text: Translatable,
    url: Translatable,
}

impl PropertyEdit {
    fn names_csv_path(base_dir: &Path) -> PathBuf {
        base_dir.join("data").join("sources").join("names.csv")
    }
    fn usages_csv_path(base_dir: &Path) -> PathBuf {
        base_dir.join("data").join("sources").join("usages.csv")
    }

    /// Reject values that would launder a pipeline-generated display string back
    /// into the curated source CSVs. Run before any writes so a stale or
    /// third-party client cannot poison `names.csv` even though the webform no
    /// longer pre-fills the editable name field with the decorated display name.
    pub(super) fn validate(&self, key: &str) -> anyhow::Result<()> {
        if let Self::Name {
            name: Some(name), ..
        } = self
        {
            let name = name.trim();
            if name.starts_with(&format!("{key} (")) || GENERATED_NAME_RE.is_match(name) {
                anyhow::bail!(
                    "the name {name:?} looks like an auto-generated display name (`<id> (<type>)`), not a human-entered name; refusing to launder it into names.csv"
                );
            }
        }
        Ok(())
    }
}

impl AppliableEdit for PropertyEdit {
    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
        match self {
            Self::Name { name, short_name } => {
                // `None` (and a blank string) means "leave this column untouched"
                // rather than "set it empty", so editing only the short name does
                // not wipe a curated name, and vice versa.
                let name = name.as_deref().map(str::trim).filter(|n| !n.is_empty());
                let short_name = short_name.as_deref();
                apply_csv_upsert_fields(
                    key,
                    &Self::names_csv_path(base_dir),
                    &[
                        Field::Set(key.to_string()),
                        name.map_or(Field::Keep, |n| Field::Set(n.to_string())),
                        short_name.map_or(Field::Keep, |s| Field::Set(s.to_string())),
                    ],
                )?;
                let mut parts = Vec::new();
                if let Some(name) = name {
                    parts.push(format!("name: `{name}`"));
                }
                if let Some(short_name) = short_name {
                    parts.push(format!("short_name: `{short_name}`"));
                }
                if parts.is_empty() {
                    anyhow::bail!("a name property edit must change the name or the short name");
                }
                Ok(parts.join(", "))
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

                // A parse failure must abort the edit: falling back to an empty
                // map here would rewrite the file with only the new link.
                let mut links: BTreeMap<String, Vec<LinkEntry>> = if yaml_path.exists() {
                    let file = File::open(&yaml_path)?;
                    serde_yaml::from_reader(file).with_context(|| {
                        format!("cannot re-serialize {} losslessly", yaml_path.display())
                    })?
                } else {
                    BTreeMap::new()
                };

                let entry = LinkEntry {
                    text: Translatable::Localized(Localized {
                        de: text_de.clone(),
                        en: text_en.clone(),
                    }),
                    url: Translatable::Plain(url.clone()),
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
    fn test_short_name_only_edit_keeps_existing_name() {
        // Regression for #3181: editing only the short name must not wipe the
        // curated name (a `None` name means "leave untouched", not "set empty").
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("names.csv"),
            format!("{NAMES_HEADER}\nalpha,Besprechungsraum,,1951\n"),
        )
        .unwrap();

        let edit = PropertyEdit::Name {
            name: None,
            short_name: Some("BR".to_string()),
        };
        let desc = edit.apply("alpha", dir.path(), "branch").unwrap();
        assert_snapshot!(desc, @"short_name: `BR`");
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r"
        id,name,short_name,arch_name
        alpha,Besprechungsraum,BR,1951
        ");
    }

    #[test]
    fn test_name_only_edit_keeps_existing_short_name() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("names.csv"),
            format!("{NAMES_HEADER}\nalpha,Old,SN,1951\n"),
        )
        .unwrap();

        let edit = PropertyEdit::Name {
            name: Some("New".to_string()),
            short_name: None,
        };
        let desc = edit.apply("alpha", dir.path(), "branch").unwrap();
        assert_snapshot!(desc, @"name: `New`");
        assert_snapshot!(fs::read_to_string(sources_dir.join("names.csv")).unwrap(), @r"
        id,name,short_name,arch_name
        alpha,New,SN,1951
        ");
    }

    #[test]
    fn test_validate_rejects_generated_display_name() {
        // The exact #3181 payload: the entry's own decorated display name.
        let edit = PropertyEdit::Name {
            name: Some("0507.01.767 (Besprechungsraum)".to_string()),
            short_name: None,
        };
        let err = edit.validate("0507.01.767").unwrap_err();
        assert!(
            err.to_string().contains("auto-generated display name"),
            "{err}"
        );
    }

    #[test]
    fn test_validate_rejects_mangled_id_prefix() {
        // The user tweaked the id prefix the form offered them (767 -> 765); the
        // result still looks like a generated `<id> (...)` string, not a name.
        let edit = PropertyEdit::Name {
            name: Some("0507.01.765 (Besprechungsraum)".to_string()),
            short_name: None,
        };
        assert!(edit.validate("0507.01.767").is_err());
    }

    #[test]
    fn test_validate_accepts_human_name() {
        let edit = PropertyEdit::Name {
            name: Some("Besprechungsraum Studentische Vertretung / SV".to_string()),
            short_name: Some("SV".to_string()),
        };
        assert!(edit.validate("0206.EG.003").is_ok());
    }

    #[test]
    fn test_validate_ignores_absent_name() {
        // A short-name-only edit carries no name and must pass validation.
        let edit = PropertyEdit::Name {
            name: None,
            short_name: Some("SV".to_string()),
        };
        assert!(edit.validate("0206.EG.003").is_ok());
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
            r"room1:
- text:
    de: Existing
    en: Existing
  url: https://old.com
",
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
    fn test_link_edit_preserves_all_real_world_shapes() {
        // The curated file mixes plain strings and `{de, en}` maps for both
        // `text` and `url`; an edit must round-trip every shape losslessly.
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("links.yaml"),
            r"'0201':
- text:
    de: Über das StudiTUM
    en: About the StudiTUM
  url:
    de: https://www.sv.tum.de/sv/studitum/
    en: https://www.sv.tum.de/en/sv/projekte/
0505.03.529A:
- text:
    de: Schreib uns doch
    en: Contact us
  url: https://tum-som.com/contact
- text: Events
  url: https://tum-som.com/events/
wzw:
- text: Speiseplan
  url:
    de: https://www.wzw.tum.de/index.php?id=33
    en: https://www.wzw.tum.de/index.php?id=33&L=1
",
        )
        .unwrap();

        let edit = PropertyEdit::Link {
            text_de: "Homepage".to_string(),
            text_en: "Homepage".to_string(),
            url: "https://example.com".to_string(),
        };
        edit.apply("0401", dir.path(), "branch").unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("links.yaml")).unwrap(), @r"
        '0201':
        - text:
            de: Über das StudiTUM
            en: About the StudiTUM
          url:
            de: https://www.sv.tum.de/sv/studitum/
            en: https://www.sv.tum.de/en/sv/projekte/
        '0401':
        - text:
            de: Homepage
            en: Homepage
          url: https://example.com
        0505.03.529A:
        - text:
            de: Schreib uns doch
            en: Contact us
          url: https://tum-som.com/contact
        - text: Events
          url: https://tum-som.com/events/
        wzw:
        - text: Speiseplan
          url:
            de: https://www.wzw.tum.de/index.php?id=33
            en: https://www.wzw.tum.de/index.php?id=33&L=1
        ");
    }

    #[test]
    fn test_link_edit_unparsable_file_aborts_without_wiping() {
        // A file the model cannot represent must fail the edit, not be
        // replaced by a file containing only the new link.
        let (dir, sources_dir) = setup();
        let unrepresentable = r"room1:
- text: Homepage
  url: https://old.com
  extra: field
";
        fs::write(sources_dir.join("links.yaml"), unrepresentable).unwrap();

        let edit = PropertyEdit::Link {
            text_de: "New".to_string(),
            text_en: "New".to_string(),
            url: "https://new.com".to_string(),
        };
        let err = edit.apply("room1", dir.path(), "branch").unwrap_err();
        assert!(err.to_string().contains("losslessly"), "{err}");
        assert_eq!(
            fs::read_to_string(sources_dir.join("links.yaml")).unwrap(),
            unrepresentable
        );
    }

    #[test]
    fn test_link_edit_unknown_language_aborts_without_wiping() {
        // Localized values are always exactly `{de, en}`; any other language
        // key is an unrepresentable shape and must abort the edit.
        let (dir, sources_dir) = setup();
        let unrepresentable = r"room1:
- text:
    de: Startseite
    en: Homepage
    fr: Accueil
  url: https://old.com
";
        fs::write(sources_dir.join("links.yaml"), unrepresentable).unwrap();

        let edit = PropertyEdit::Link {
            text_de: "New".to_string(),
            text_en: "New".to_string(),
            url: "https://new.com".to_string(),
        };
        let err = edit.apply("room1", dir.path(), "branch").unwrap_err();
        assert!(err.to_string().contains("losslessly"), "{err}");
        assert_eq!(
            fs::read_to_string(sources_dir.join("links.yaml")).unwrap(),
            unrepresentable
        );
    }

    #[test]
    fn test_link_edit_treats_empty_file_like_missing_file() {
        // An empty file holds no links that could be lost, so the hard
        // parse-failure guard must not reject it.
        let (dir, sources_dir) = setup();
        fs::write(sources_dir.join("links.yaml"), "").unwrap();

        let edit = PropertyEdit::Link {
            text_de: "Homepage".to_string(),
            text_en: "Homepage".to_string(),
            url: "https://example.com".to_string(),
        };
        edit.apply("room1", dir.path(), "branch").unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("links.yaml")).unwrap(), @r"
        room1:
        - text:
            de: Homepage
            en: Homepage
          url: https://example.com
        ");
    }

    #[test]
    fn test_curated_links_yaml_is_representable() {
        // Canary: if the curated file gains a shape the model cannot
        // represent, link edits start failing at submit time.
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/sources/links.yaml");
        let links: BTreeMap<String, Vec<LinkEntry>> =
            serde_yaml::from_reader(File::open(path).unwrap()).unwrap();
        assert!(!links.is_empty());
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
