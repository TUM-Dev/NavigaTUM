use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::coordinate::Coordinate;
use super::areatree::AreatreeKind;
use super::validation::{AdditionError, AdditionVariant, CollisionSource, RepoSnapshot};
use super::{AppliableAddition, AppliedAddition};

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct RoomLink {
    pub text_de: String,
    pub text_en: String,
    pub url: String,
}

const MAX_NAME_LEN: usize = 200;
// Kept in sync with `ALLOWED_ROOMCODE_CHARS` in `data/processors/tumonline.py`.
fn is_allowed_roomcode_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '-')
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct NewRoom {
    pub parent_building_id: String,
    pub alt_name: String,
    pub arch_name: String,
    pub usage_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seats: Option<Seats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<RoomAddress>,
    pub coords: Coordinate,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<RoomLink>,
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct Seats {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sitting: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standing: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wheelchair: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct RoomAddress {
    pub place: String,
    pub street: String,
    pub zip_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlRoomAddition {
    room_key: String,
    #[serde(flatten)]
    new_room: NewRoom,
}

fn validate_room_code(key: &str) -> Result<&str, AdditionError> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 3 {
        return Err(AdditionError::BadRoomCode(
            key.to_string(),
            "expected 3 dot-separated segments",
        ));
    }
    if !key.chars().all(is_allowed_roomcode_char) {
        return Err(AdditionError::BadRoomCode(
            key.to_string(),
            "contains disallowed characters",
        ));
    }
    if parts.iter().any(|p| p.is_empty()) {
        return Err(AdditionError::BadRoomCode(key.to_string(), "empty segment"));
    }
    parts
        .first()
        .copied()
        .ok_or_else(|| AdditionError::BadRoomCode(key.to_string(), "missing first segment"))
}

fn is_arch_name_valid(s: &str) -> bool {
    let Some((name, building)) = s.split_once('@') else {
        return false;
    };
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return false;
    }
    building.len() == 4 && building.chars().all(|c| c.is_ascii_digit())
}

impl AppliableAddition for NewRoom {
    fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        const ALLOWED_PARENT_KINDS: &[AreatreeKind] =
            &[AreatreeKind::Building, AreatreeKind::JoinedBuilding];

        let prefix = validate_room_code(key)?;
        if prefix != self.parent_building_id {
            return Err(AdditionError::PrefixMismatch {
                code: key.to_string(),
                got: prefix.to_string(),
                want: self.parent_building_id.clone(),
            });
        }
        let parent = snap
            .areatree
            .find(&self.parent_building_id)
            .ok_or_else(|| AdditionError::UnknownParent {
                parent: self.parent_building_id.clone(),
            })?;
        if !ALLOWED_PARENT_KINDS.contains(&parent.kind) {
            return Err(AdditionError::WrongParentType {
                parent: self.parent_building_id.clone(),
                actual: parent.kind,
                kind: AdditionVariant::Room,
                expected: ALLOWED_PARENT_KINDS,
            });
        }
        if !snap.usage_ids.contains(&self.usage_id) {
            return Err(AdditionError::UnknownUsageId(self.usage_id));
        }
        if !is_arch_name_valid(&self.arch_name) {
            return Err(AdditionError::BadArchName(self.arch_name.clone()));
        }
        if self.alt_name.is_empty() || self.alt_name.len() > MAX_NAME_LEN {
            return Err(AdditionError::BadName {
                len: self.alt_name.len(),
                max: MAX_NAME_LEN,
            });
        }
        if snap.tumonline_room_codes.contains(key) {
            return Err(AdditionError::IdCollision {
                id: key.to_string(),
                at: CollisionSource::TumonlineRooms,
            });
        }
        if snap.user_added_room_codes.contains(key) {
            return Err(AdditionError::IdCollision {
                id: key.to_string(),
                at: CollisionSource::UserRoomAdditions,
            });
        }
        Ok(())
    }

    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<AppliedAddition> {
        let yaml_path = base_dir
            .join("data")
            .join("sources")
            .join("15_patches-rooms_tumonline.yaml");
        let raw = fs::read_to_string(&yaml_path)?;
        // Round-trip via untyped YAML so the file's other top-level keys (`patches`, …)
        // survive untouched.
        let mut as_map: BTreeMap<String, serde_yaml::Value> = if raw.trim().is_empty() {
            BTreeMap::new()
        } else {
            serde_yaml::from_str(&raw).unwrap_or_default()
        };

        let mut additions: Vec<YamlRoomAddition> = match as_map.remove("additions") {
            Some(v) if !v.is_null() => serde_yaml::from_value(v).unwrap_or_default(),
            _ => Vec::new(),
        };
        additions.push(YamlRoomAddition {
            room_key: key.to_string(),
            new_room: self.clone(),
        });
        as_map.insert("additions".to_string(), serde_yaml::to_value(&additions)?);

        let out = serde_yaml::to_string(&as_map)?;
        fs::write(&yaml_path, out)?;

        self.coords.apply_to_csv(key, base_dir)?;

        Ok(AppliedAddition::created(format!(
            "new room `{key}` ({alt}, arch_name `{arch}`, usage_id {uid}) @ {coords:?}",
            alt = self.alt_name,
            arch = self.arch_name,
            uid = self.usage_id,
            coords = self.coords,
        )))
    }

    fn kind_label(&self) -> &'static str {
        "room"
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

    fn snapshot_with(areatree: &str) -> RepoSnapshot {
        RepoSnapshot {
            areatree: AreatreeIndex::parse(areatree).unwrap(),
            tumonline_room_codes: HashSet::from(["0101.01.999".to_string()]),
            user_added_room_codes: HashSet::new(),
            poi_keys: HashSet::new(),
            usage_ids: HashSet::from([12]),
            org_ids: HashSet::new(),
            event_row_counts: HashMap::new(),
            now: chrono::Utc::now(),
        }
    }

    fn sample_coord() -> Coordinate {
        serde_json::from_value(serde_json::json!({"lat": 48.262, "lon": 11.668})).unwrap()
    }

    fn sample_room() -> NewRoom {
        NewRoom {
            parent_building_id: "5117".to_string(),
            alt_name: "Testraum".to_string(),
            arch_name: "EG103@5117".to_string(),
            usage_id: 12,
            seats: None,
            floor_type: None,
            floor_level: None,
            address: None,
            coords: sample_coord(),
            links: vec![],
        }
    }

    fn ar() -> &'static str {
        "\
:Standorte:root[root]
  0:Stammgelände:stammgelaende[campus]
    51:M-Gelände:m
      5117:Foo:5117
"
    }

    type Mutate = fn(&mut NewRoom, &mut RepoSnapshot);
    type Check = fn(&AdditionError) -> bool;

    // One case per `AdditionError` variant the room validator can emit; new rules added to
    // `validate` should land here as a new `#[case]`.
    #[rstest]
    #[case::bad_room_code(
        (|_r, _s| {}) as Mutate,
        "5117.EG",
        (|e| matches!(e, AdditionError::BadRoomCode(_, _))) as Check
    )]
    #[case::prefix_mismatch(
        (|_r, _s| {}) as Mutate,
        "0101.EG.103",
        (|e| matches!(e, AdditionError::PrefixMismatch { .. })) as Check
    )]
    #[case::unknown_parent(
        (|r, _s| { r.parent_building_id = "9999".to_string(); }) as Mutate,
        "9999.EG.103",
        (|e| matches!(e, AdditionError::UnknownParent { .. })) as Check
    )]
    #[case::wrong_parent_type(
        (|r, _s| { r.parent_building_id = "m".to_string(); }) as Mutate,
        "m.EG.103",
        (|e| matches!(e, AdditionError::WrongParentType { .. })) as Check
    )]
    #[case::unknown_usage_id(
        (|r, _s| { r.usage_id = 999; }) as Mutate,
        "5117.EG.103",
        (|e| matches!(e, AdditionError::UnknownUsageId(999))) as Check
    )]
    #[case::bad_arch_name(
        (|r, _s| { r.arch_name = "bad-arch-name".to_string(); }) as Mutate,
        "5117.EG.103",
        (|e| matches!(e, AdditionError::BadArchName(_))) as Check
    )]
    #[case::id_collision_with_tumonline(
        (|_r, s| { s.tumonline_room_codes.insert("5117.EG.103".to_string()); }) as Mutate,
        "5117.EG.103",
        (|e| matches!(e, AdditionError::IdCollision { .. })) as Check
    )]
    fn validate_failure_cases(#[case] mutate: Mutate, #[case] key: &str, #[case] check: Check) {
        let mut r = sample_room();
        let mut s = snapshot_with(ar());
        mutate(&mut r, &mut s);
        let err = r.validate(key, &s).unwrap_err();
        assert!(check(&err), "got: {err}");
    }

    #[test]
    fn validate_happy_path() {
        sample_room()
            .validate("5117.EG.103", &snapshot_with(ar()))
            .unwrap();
    }

    fn setup_apply_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources).unwrap();
        fs::write(
            sources.join("15_patches-rooms_tumonline.yaml"),
            "patches: []\n",
        )
        .unwrap();
        fs::write(sources.join("coordinates.csv"), "id,lat,lon\n").unwrap();
        dir
    }

    #[test]
    fn apply_writes_yaml_and_coordinates() {
        let dir = setup_apply_dir();
        let summary = sample_room()
            .apply("5117.EG.103", dir.path(), "branch")
            .unwrap()
            .summary;
        assert_snapshot!(
            summary,
            @"new room `5117.EG.103` (Testraum, arch_name `EG103@5117`, usage_id 12) @ Coordinate { lat: 48.262, lon: 11.668 }"
        );
        let yaml = fs::read_to_string(
            dir.path()
                .join("data/sources/15_patches-rooms_tumonline.yaml"),
        )
        .unwrap();
        assert_snapshot!(yaml, @r"
        additions:
        - room_key: 5117.EG.103
          parent_building_id: '5117'
          alt_name: Testraum
          arch_name: EG103@5117
          usage_id: 12
          coords:
            lat: 48.262
            lon: 11.668
        patches: []
        ");
        let coords = fs::read_to_string(dir.path().join("data/sources/coordinates.csv")).unwrap();
        assert_snapshot!(coords, @r"
        id,lat,lon
        5117.EG.103,48.262,11.668
        ");
    }

    #[test]
    fn missing_coords_fails_to_deserialize() {
        let json = serde_json::json!({
            "parent_building_id": "5117",
            "alt_name": "x",
            "arch_name": "EG@5117",
            "usage_id": 12
        });
        let err = serde_json::from_value::<NewRoom>(json).unwrap_err();
        assert!(err.to_string().contains("coords"), "got: {err}");
    }
}
