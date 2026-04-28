//! New buildings/areas — written by inserting a line into `config.areatree` (and writing one
//! coordinate row per building prefix into `coordinates.csv`).
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::coordinate::Coordinate;
use super::AppliableAddition;
use super::areatree::{AreatreeKind, format_line, insert_under};
use super::validation::{AdditionError, RepoSnapshot};

const MAX_NAME_LEN: usize = 200;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildingKind {
    Building,
    JoinedBuilding,
    Area,
}

impl BuildingKind {
    fn as_areatree_kind(self) -> AreatreeKind {
        match self {
            Self::Building => AreatreeKind::Building,
            Self::JoinedBuilding => AreatreeKind::JoinedBuilding,
            Self::Area => AreatreeKind::Area,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct NewBuilding {
    pub parent_id: String,
    /// What kind of node this is. Renamed away from `kind` because the [`super::Addition`]
    /// enum already uses `kind` as its serde tag.
    #[serde(rename = "node_kind")]
    pub kind: BuildingKind,
    pub building_prefixes: Vec<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coords: Option<Coordinate>,
}

impl NewBuilding {
    /// The id this entry will end up with in the areatree.
    fn effective_id(&self) -> Option<String> {
        if let Some(ref id) = self.internal_id {
            return Some(id.clone());
        }
        if self.building_prefixes.len() == 1 {
            return Some(self.building_prefixes[0].clone());
        }
        None
    }
}

impl AppliableAddition for NewBuilding {
    fn validate(&self, _key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        match self.kind {
            BuildingKind::Building => {
                if self.building_prefixes.len() != 1 {
                    return Err(AdditionError::BuildingNeedsExactlyOnePrefix(
                        self.building_prefixes.len(),
                    ));
                }
                if self.coords.is_none() {
                    return Err(AdditionError::MissingCoords("building"));
                }
            }
            BuildingKind::JoinedBuilding => {
                if self.building_prefixes.len() < 2 {
                    return Err(AdditionError::JoinedBuildingNeedsMultiplePrefixes(
                        self.building_prefixes.len(),
                    ));
                }
                if self.coords.is_none() {
                    return Err(AdditionError::MissingCoords("joined_building"));
                }
            }
            BuildingKind::Area => {
                // Area without prefixes is fine; coords optional.
            }
        }
        for prefix in &self.building_prefixes {
            if prefix.len() != 4 || !prefix.chars().all(|c| c.is_ascii_digit()) {
                return Err(AdditionError::BadBuildingPrefix(prefix.clone()));
            }
            if snap.areatree.contains_b_prefix(prefix) || snap.areatree.contains_id(prefix) {
                return Err(AdditionError::BuildingPrefixCollision(prefix.clone()));
            }
        }
        let parent =
            snap.areatree
                .find(&self.parent_id)
                .ok_or_else(|| AdditionError::UnknownParent {
                    parent: self.parent_id.clone(),
                })?;
        let allowed: &[&str] = &["root", "site", "campus", "area"];
        if !allowed.contains(&parent.kind.as_str()) {
            return Err(AdditionError::WrongParentType {
                parent: self.parent_id.clone(),
                actual: parent.kind.as_str().to_string(),
                kind: "building",
                expected: allowed,
            });
        }

        let effective_id = self
            .effective_id()
            .ok_or_else(|| AdditionError::BadId(String::new()))?;
        if snap.areatree.contains_id(&effective_id) {
            return Err(AdditionError::IdCollision(effective_id, "config.areatree"));
        }
        if let Some(ref vid) = self.visible_id
            && snap.areatree.contains_visible_id(vid)
        {
            return Err(AdditionError::VisibleIdCollision(vid.clone()));
        }
        if self.name.is_empty() || self.name.len() > MAX_NAME_LEN {
            return Err(AdditionError::BadName {
                len: self.name.len(),
                max: MAX_NAME_LEN,
            });
        }
        Ok(())
    }

    fn apply(&self, _key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
        let areatree_path = base_dir
            .join("data")
            .join("processors")
            .join("areatree")
            .join("config.areatree");
        let content = fs::read_to_string(&areatree_path)?;

        let kind_at = self.kind.as_areatree_kind();
        let effective_id = self
            .effective_id()
            .ok_or_else(|| anyhow::anyhow!("internal_id required for multi-prefix building"))?;

        let line = format_line(
            &self.building_prefixes,
            &self.name,
            self.short_name.as_deref(),
            &effective_id,
            self.visible_id.as_deref(),
            &kind_at,
        );
        let updated = insert_under(&content, &self.parent_id, &effective_id, &line)?;
        fs::write(&areatree_path, updated)?;

        if let Some(coord) = self.coords {
            coord.apply_to_csv(&effective_id, base_dir)?;
            for prefix in &self.building_prefixes {
                if prefix != &effective_id {
                    coord.apply_to_csv(prefix, base_dir)?;
                }
            }
        }

        let geojson = serde_json::json!({
            "type": "Feature",
            "geometry": match self.coords {
                Some(c) => serde_json::json!({
                    "type": "Point",
                    "coordinates": [c.lon, c.lat]
                }),
                None => serde_json::Value::Null,
            },
            "properties": {
                "kind": "new-building",
                "id": effective_id,
                "name": self.name,
                "node_kind": self.kind,
                "parent_id": self.parent_id,
                "building_prefixes": self.building_prefixes,
            }
        });
        let pretty = serde_json::to_string_pretty(&geojson).unwrap_or_else(|_| geojson.to_string());
        Ok(format!("```geojson\n{pretty}\n```"))
    }

    fn kind_label(&self) -> &'static str {
        "building"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
    use std::collections::HashSet;
    use std::fs;

    use super::super::areatree::AreatreeIndex;
    use super::*;

    fn ar() -> &'static str {
        "\
:Standorte:root[root]
  0:Stammgelände:stammgelaende[campus]
    01:Nordgelände:nordgelaende
      0101:N1:0101,n1
"
    }

    fn snapshot() -> RepoSnapshot {
        RepoSnapshot {
            areatree: AreatreeIndex::parse(ar()).unwrap(),
            tumonline_room_codes: HashSet::new(),
            user_added_room_codes: HashSet::new(),
            poi_keys: HashSet::new(),
            usage_ids: HashSet::new(),
            coord_ids: HashSet::new(),
        }
    }

    fn coord() -> Coordinate {
        serde_json::from_value(serde_json::json!({"lat": 48.0, "lon": 11.0})).unwrap()
    }

    #[test]
    fn validate_building_happy() {
        let b = NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec!["0103".to_string()],
            name: "New Bldg".to_string(),
            short_name: Some("NB".to_string()),
            internal_id: None,
            visible_id: Some("nb".to_string()),
            coords: Some(coord()),
        };
        b.validate("0103", &snapshot()).unwrap();
    }

    #[test]
    fn validate_building_needs_one_prefix() {
        let b = NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec![],
            name: "x".to_string(),
            short_name: None,
            internal_id: None,
            visible_id: None,
            coords: Some(coord()),
        };
        let err = b.validate("x", &snapshot()).unwrap_err();
        assert!(matches!(
            err,
            AdditionError::BuildingNeedsExactlyOnePrefix(0)
        ));
    }

    #[test]
    fn validate_joined_needs_multi_prefix() {
        let b = NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::JoinedBuilding,
            building_prefixes: vec!["1500".to_string()],
            name: "x".to_string(),
            short_name: None,
            internal_id: Some("1500".to_string()),
            visible_id: None,
            coords: Some(coord()),
        };
        let err = b.validate("x", &snapshot()).unwrap_err();
        assert!(matches!(
            err,
            AdditionError::JoinedBuildingNeedsMultiplePrefixes(1)
        ));
    }

    #[test]
    fn validate_building_missing_coords() {
        let b = NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec!["0103".to_string()],
            name: "x".to_string(),
            short_name: None,
            internal_id: None,
            visible_id: None,
            coords: None,
        };
        let err = b.validate("x", &snapshot()).unwrap_err();
        assert!(matches!(err, AdditionError::MissingCoords(_)));
    }

    #[test]
    fn validate_bad_prefix_format() {
        let b = NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec!["abc".to_string()],
            name: "x".to_string(),
            short_name: None,
            internal_id: None,
            visible_id: None,
            coords: Some(coord()),
        };
        let err = b.validate("x", &snapshot()).unwrap_err();
        assert!(matches!(err, AdditionError::BadBuildingPrefix(_)));
    }

    #[test]
    fn validate_prefix_collision() {
        let b = NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec!["0101".to_string()],
            name: "x".to_string(),
            short_name: None,
            internal_id: None,
            visible_id: None,
            coords: Some(coord()),
        };
        let err = b.validate("x", &snapshot()).unwrap_err();
        assert!(matches!(err, AdditionError::BuildingPrefixCollision(_)));
    }

    #[test]
    fn validate_wrong_parent_type() {
        let b = NewBuilding {
            parent_id: "0101".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec!["0103".to_string()],
            name: "x".to_string(),
            short_name: None,
            internal_id: None,
            visible_id: None,
            coords: Some(coord()),
        };
        let err = b.validate("x", &snapshot()).unwrap_err();
        assert!(matches!(err, AdditionError::WrongParentType { .. }));
    }

    #[test]
    fn apply_inserts_line() {
        let dir = tempfile::tempdir().unwrap();
        let proc = dir.path().join("data").join("processors").join("areatree");
        fs::create_dir_all(&proc).unwrap();
        fs::write(proc.join("config.areatree"), ar()).unwrap();
        let sources = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources).unwrap();
        fs::write(sources.join("coordinates.csv"), "id,lat,lon\n").unwrap();

        let b = NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec!["0103".to_string()],
            name: "New Bldg".to_string(),
            short_name: Some("NB".to_string()),
            internal_id: None,
            visible_id: Some("nb".to_string()),
            coords: Some(coord()),
        };
        let summary = b.apply("0103", dir.path(), "branch").unwrap();
        assert!(summary.contains("new-building"));
        let updated = fs::read_to_string(proc.join("config.areatree")).unwrap();
        assert!(updated.contains("0103:New Bldg|NB:0103,nb"));
        let coords = fs::read_to_string(sources.join("coordinates.csv")).unwrap();
        assert!(
            coords.contains("0103,48,11") || coords.contains("0103,48.0,11.0"),
            "coordinates.csv unexpected: {coords}"
        );
    }
}
