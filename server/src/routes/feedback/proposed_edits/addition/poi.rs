//! New POIs — appended to `21_pois.yaml` (a flat `BTreeMap<String, PoiEntry>`).
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::coordinate::Coordinate;
use super::AppliableAddition;
use super::validation::{AdditionError, RepoSnapshot};

const MAX_NAME_LEN: usize = 200;
const MAX_KEY_LEN: usize = 64;

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct TranslatableStr {
    pub de: String,
    pub en: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct PoiLink {
    pub text: TranslatableStr,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct GenericProp {
    pub name: TranslatableStr,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct NewPoi {
    pub parent: String,
    pub name: String,
    pub usage_name: String,
    pub coords: Coordinate,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<PoiLink>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<TranslatableStr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_props: Vec<GenericProp>,
}

fn is_valid_poi_key(key: &str) -> bool {
    let bytes = key.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_KEY_LEN {
        return false;
    }
    let first = bytes[0];
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-'))
}

#[derive(Debug, Serialize)]
struct YamlPoiEntry {
    parent: String,
    name: String,
    usage: YamlUsage,
    coords: Coordinate,
    #[serde(skip_serializing_if = "YamlProps::is_empty")]
    props: YamlProps,
}

#[derive(Debug, Serialize)]
struct YamlUsage {
    name: String,
}

#[derive(Debug, Serialize, Default)]
struct YamlProps {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<PoiLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<TranslatableStr>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    generic: Vec<GenericProp>,
}

impl YamlProps {
    fn is_empty(&self) -> bool {
        self.links.is_empty() && self.comment.is_none() && self.generic.is_empty()
    }
}

impl AppliableAddition for NewPoi {
    fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        if !is_valid_poi_key(key) {
            return Err(AdditionError::BadId(key.to_string()));
        }
        if self.name.is_empty() || self.name.len() > MAX_NAME_LEN {
            return Err(AdditionError::BadName {
                len: self.name.len(),
                max: MAX_NAME_LEN,
            });
        }
        if self.usage_name.is_empty() {
            return Err(AdditionError::BadUsageName);
        }
        if snap.poi_keys.contains(key) {
            return Err(AdditionError::IdCollision(key.to_string(), "21_pois.yaml"));
        }
        let parent_known = snap.areatree.contains_id(&self.parent)
            || snap.tumonline_room_codes.contains(&self.parent)
            || snap.user_added_room_codes.contains(&self.parent)
            || snap.poi_keys.contains(&self.parent);
        if !parent_known {
            return Err(AdditionError::UnknownParent {
                parent: self.parent.clone(),
            });
        }
        Ok(())
    }

    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
        let yaml_path = base_dir.join("data").join("sources").join("21_pois.yaml");
        let raw = fs::read_to_string(&yaml_path).unwrap_or_default();
        let mut map: BTreeMap<String, serde_yaml::Value> = if raw.trim().is_empty() {
            BTreeMap::new()
        } else {
            serde_yaml::from_str(&raw).unwrap_or_default()
        };
        let entry = YamlPoiEntry {
            parent: self.parent.clone(),
            name: self.name.clone(),
            usage: YamlUsage {
                name: self.usage_name.clone(),
            },
            coords: self.coords,
            props: YamlProps {
                links: self.links.clone(),
                comment: self.comment.clone(),
                generic: self.generic_props.clone(),
            },
        };
        map.insert(key.to_string(), serde_yaml::to_value(&entry)?);
        let out = serde_yaml::to_string(&map)?;
        fs::write(&yaml_path, out)?;

        Ok(format!(
            "new POI `{key}` ({name}, usage `{usage}`, parent `{parent}`)",
            name = self.name,
            usage = self.usage_name,
            parent = self.parent,
        ))
    }

    fn kind_label(&self) -> &'static str {
        "poi"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
    use std::collections::HashSet;
    use std::fs;

    use super::super::areatree::AreatreeIndex;
    use super::*;

    fn snapshot() -> RepoSnapshot {
        RepoSnapshot {
            areatree: AreatreeIndex::parse(":Standorte:root[root]\n  0:Stamm:0[campus]\n").unwrap(),
            tumonline_room_codes: HashSet::from(["5101.EG.917".to_string()]),
            user_added_room_codes: HashSet::new(),
            poi_keys: HashSet::from(["existing-poi".to_string()]),
            usage_ids: HashSet::new(),
            coord_ids: HashSet::new(),
        }
    }

    fn sample_coord() -> Coordinate {
        serde_json::from_value(serde_json::json!({"lat": 48.262, "lon": 11.668})).unwrap()
    }

    fn sample_poi() -> NewPoi {
        NewPoi {
            parent: "5101.EG.917".to_string(),
            name: "Validierungsautomat 99".to_string(),
            usage_name: "Validierungsautomat".to_string(),
            coords: sample_coord(),
            links: vec![],
            comment: None,
            generic_props: vec![],
        }
    }

    #[test]
    fn missing_coords_fails_to_deserialize() {
        let json = serde_json::json!({
            "parent": "0501",
            "name": "x",
            "usage_name": "x"
        });
        let err = serde_json::from_value::<NewPoi>(json).unwrap_err();
        assert!(err.to_string().contains("coords"), "got: {err}");
    }

    #[test]
    fn validate_happy() {
        sample_poi()
            .validate("validierungsautomat-99", &snapshot())
            .unwrap();
    }

    #[test]
    fn validate_bad_key() {
        let err = sample_poi().validate("BadKey!", &snapshot()).unwrap_err();
        assert!(matches!(err, AdditionError::BadId(_)));
    }

    #[test]
    fn validate_id_collision() {
        let err = sample_poi()
            .validate("existing-poi", &snapshot())
            .unwrap_err();
        assert!(matches!(err, AdditionError::IdCollision(_, _)));
    }

    #[test]
    fn validate_unknown_parent() {
        let mut p = sample_poi();
        p.parent = "nonexistent".to_string();
        let err = p.validate("new-poi", &snapshot()).unwrap_err();
        assert!(matches!(err, AdditionError::UnknownParent { .. }));
    }

    #[test]
    fn apply_writes_into_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources).unwrap();
        fs::write(
            sources.join("21_pois.yaml"),
            "existing-poi:\n  parent: \"x\"\n  name: \"x\"\n  usage:\n    name: \"x\"\n",
        )
        .unwrap();

        let p = sample_poi();
        let summary = p
            .apply("validierungsautomat-99", dir.path(), "branch")
            .unwrap();
        assert!(summary.contains("validierungsautomat-99"));
        let written = fs::read_to_string(sources.join("21_pois.yaml")).unwrap();
        assert!(written.contains("validierungsautomat-99:"));
        assert!(written.contains("existing-poi:")); // pre-existing entry preserved
        assert!(written.contains("Validierungsautomat 99"));
    }
}
