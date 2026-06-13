use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::coordinate::Coordinate;
use super::areatree::{AreatreeKind, format_line, insert_under};
use super::validation::{AdditionError, AdditionVariant, CollisionSource, RepoSnapshot};
use super::{AppliableAddition, AppliedAddition};

const MAX_NAME_LEN: usize = 200;

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    utoipa::ToSchema,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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
    // Renamed at the wire layer so it doesn't collide with the outer `Addition` serde tag.
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
    pub coords: Coordinate,
}

impl NewBuilding {
    fn effective_id(&self) -> Option<String> {
        if let Some(ref id) = self.internal_id {
            return Some(id.clone());
        }
        if let [only] = self.building_prefixes.as_slice() {
            return Some(only.clone());
        }
        None
    }
}

impl AppliableAddition for NewBuilding {
    fn validate(&self, _key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        const ALLOWED_PARENT_KINDS: &[AreatreeKind] = &[
            AreatreeKind::Root,
            AreatreeKind::Site,
            AreatreeKind::Campus,
            AreatreeKind::Area,
        ];

        match self.kind {
            BuildingKind::Building if self.building_prefixes.len() != 1 => {
                return Err(AdditionError::BuildingNeedsExactlyOnePrefix(
                    self.building_prefixes.len(),
                ));
            }
            BuildingKind::JoinedBuilding if self.building_prefixes.len() < 2 => {
                return Err(AdditionError::JoinedBuildingNeedsMultiplePrefixes(
                    self.building_prefixes.len(),
                ));
            }
            _ => {}
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
        if !ALLOWED_PARENT_KINDS.contains(&parent.kind) {
            return Err(AdditionError::WrongParentType {
                parent: self.parent_id.clone(),
                actual: parent.kind,
                kind: AdditionVariant::Building,
                expected: ALLOWED_PARENT_KINDS,
            });
        }

        let effective_id = self
            .effective_id()
            .ok_or_else(|| AdditionError::BadId(String::new()))?;
        if snap.areatree.contains_id(&effective_id) {
            return Err(AdditionError::IdCollision {
                id: effective_id,
                at: CollisionSource::Areatree,
            });
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

    fn apply(&self, _key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<AppliedAddition> {
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
            kind_at,
        );
        let updated = insert_under(&content, &self.parent_id, &effective_id, &line)?;
        fs::write(&areatree_path, updated)?;

        self.coords.apply_to_csv(&effective_id, base_dir)?;
        for prefix in &self.building_prefixes {
            if prefix != &effective_id {
                self.coords.apply_to_csv(prefix, base_dir)?;
            }
        }

        Ok(AppliedAddition::created(
            self.coords.fenced_geojson_feature(&serde_json::json!({
                "kind": "new-building",
                "id": effective_id,
                "name": self.name,
                "node_kind": self.kind,
                "parent_id": self.parent_id,
                "building_prefixes": self.building_prefixes,
            })),
        ))
    }

    fn kind_label(&self) -> &'static str {
        "building"
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
    use std::collections::{HashMap, HashSet};
    use std::fs;

    use insta::assert_snapshot;
    use rstest::rstest;

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
            org_ids: HashSet::new(),
            event_row_counts: HashMap::new(),
            now: chrono::Utc::now(),
        }
    }

    fn coord() -> Coordinate {
        serde_json::from_value(serde_json::json!({"lat": 48.0, "lon": 11.0})).unwrap()
    }

    fn sample_building() -> NewBuilding {
        NewBuilding {
            parent_id: "nordgelaende".to_string(),
            kind: BuildingKind::Building,
            building_prefixes: vec!["0103".to_string()],
            name: "New Bldg".to_string(),
            short_name: Some("NB".to_string()),
            internal_id: None,
            visible_id: Some("nb".to_string()),
            coords: coord(),
        }
    }

    #[test]
    fn validate_building_happy() {
        sample_building().validate("0103", &snapshot()).unwrap();
    }

    type Mutate = fn(&mut NewBuilding);
    type Check = fn(&AdditionError) -> bool;

    #[rstest]
    #[case::needs_one_prefix(
        (|b| { b.building_prefixes.clear(); }) as Mutate,
        (|e| matches!(e, AdditionError::BuildingNeedsExactlyOnePrefix(0))) as Check
    )]
    #[case::joined_needs_multi_prefix(
        (|b| {
            b.kind = BuildingKind::JoinedBuilding;
            b.internal_id = Some("1500".to_string());
            b.building_prefixes = vec!["1500".to_string()];
        }) as Mutate,
        (|e| matches!(e, AdditionError::JoinedBuildingNeedsMultiplePrefixes(1))) as Check
    )]
    #[case::bad_prefix_format(
        (|b| { b.building_prefixes = vec!["abc".to_string()]; }) as Mutate,
        (|e| matches!(e, AdditionError::BadBuildingPrefix(_))) as Check
    )]
    #[case::prefix_collision(
        (|b| { b.building_prefixes = vec!["0101".to_string()]; }) as Mutate,
        (|e| matches!(e, AdditionError::BuildingPrefixCollision(_))) as Check
    )]
    #[case::wrong_parent_type(
        (|b| { b.parent_id = "0101".to_string(); }) as Mutate,
        (|e| matches!(e, AdditionError::WrongParentType { .. })) as Check
    )]
    fn validate_failure_cases(#[case] mutate: Mutate, #[case] check: Check) {
        let mut b = sample_building();
        mutate(&mut b);
        let err = b.validate("x", &snapshot()).unwrap_err();
        assert!(check(&err), "got: {err}");
    }

    #[test]
    fn missing_coords_fails_to_deserialize() {
        let json = serde_json::json!({
            "parent_id": "nordgelaende",
            "node_kind": "building",
            "building_prefixes": ["0103"],
            "name": "x"
        });
        let err = serde_json::from_value::<NewBuilding>(json).unwrap_err();
        assert!(err.to_string().contains("coords"), "got: {err}");
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

        let summary = sample_building()
            .apply("0103", dir.path(), "branch")
            .unwrap()
            .summary;
        assert_snapshot!(summary, @r#"
        ```geojson
        {
          "geometry": {
            "coordinates": [
              11.0,
              48.0
            ],
            "type": "Point"
          },
          "properties": {
            "building_prefixes": [
              "0103"
            ],
            "id": "0103",
            "kind": "new-building",
            "name": "New Bldg",
            "node_kind": "building",
            "parent_id": "nordgelaende"
          },
          "type": "Feature"
        }
        ```
        "#);
        let areatree = fs::read_to_string(proc.join("config.areatree")).unwrap();
        assert_snapshot!(areatree, @r"
        :Standorte:root[root]
          0:Stammgelände:stammgelaende[campus]
            01:Nordgelände:nordgelaende
              0101:N1:0101,n1
              0103:New Bldg|NB:0103,nb
        ");
        let coords = fs::read_to_string(sources.join("coordinates.csv")).unwrap();
        assert_snapshot!(coords, @r"
        id,lat,lon
        0103,48,11
        ");
    }
}
