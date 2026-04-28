//! New rooms — written into the `additions:` block of `15_patches-rooms_tumonline.yaml`,
//! plus an optional coordinate written through the existing coordinate writer.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::coordinate::Coordinate;
use super::AppliableAddition;
use super::areatree::AreatreeKind;
use super::validation::{AdditionError, RepoSnapshot};

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct RoomLink {
    pub text_de: String,
    pub text_en: String,
    pub url: String,
}

const MAX_NAME_LEN: usize = 200;
/// Allowed characters for a room key (matches `ALLOWED_ROOMCODE_CHARS` in tumonline.py).
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coords: Option<Coordinate>,
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

/// What we serialize into the `additions:` YAML list. Holds the room key (`room_key`) inline so
/// the Python data processor reads it as the dict key, plus all `NewRoom` fields.
#[derive(Debug, Serialize, Deserialize)]
struct YamlRoomAddition {
    room_key: String,
    #[serde(flatten)]
    new_room: NewRoom,
}

#[derive(Debug)]
pub struct RoomCode {
    pub building_prefix: String,
}

impl RoomCode {
    pub fn parse(key: &str) -> Result<Self, AdditionError> {
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
        Ok(Self {
            building_prefix: parts[0].to_string(),
        })
    }
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
        let code = RoomCode::parse(key)?;
        if code.building_prefix != self.parent_building_id {
            return Err(AdditionError::PrefixMismatch {
                code: key.to_string(),
                got: code.building_prefix,
                want: self.parent_building_id.clone(),
            });
        }
        let parent = snap
            .areatree
            .find(&self.parent_building_id)
            .ok_or_else(|| AdditionError::UnknownParent {
                parent: self.parent_building_id.clone(),
            })?;
        if !matches!(
            parent.kind,
            AreatreeKind::Building | AreatreeKind::JoinedBuilding
        ) {
            return Err(AdditionError::WrongParentType {
                parent: self.parent_building_id.clone(),
                actual: parent.kind.as_str().to_string(),
                kind: "room",
                expected: &["building", "joined_building"],
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
            return Err(AdditionError::IdCollision(
                key.to_string(),
                "rooms_tumonline.csv",
            ));
        }
        if snap.user_added_room_codes.contains(key) {
            return Err(AdditionError::IdCollision(
                key.to_string(),
                "15_patches-rooms_tumonline.yaml additions",
            ));
        }
        Ok(())
    }

    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
        let yaml_path = base_dir
            .join("data")
            .join("sources")
            .join("15_patches-rooms_tumonline.yaml");
        let raw = fs::read_to_string(&yaml_path)?;
        // Preserve unknown top-level keys via a flexible deserializer.
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

        // Re-serialize. Insertion order is not preserved by BTreeMap (alphabetical), but the
        // additions block is identified by its own key so that's fine.
        let out = serde_yaml::to_string(&as_map)?;
        fs::write(&yaml_path, out)?;

        if let Some(coord) = self.coords {
            coord.apply_to_csv(key, base_dir)?;
        }

        let coord_str = self
            .coords
            .as_ref()
            .map(|c| format!(" @ {c:?}"))
            .unwrap_or_default();
        Ok(format!(
            "new room `{key}` ({alt}, arch_name `{arch}`, usage_id {uid}){coord_str}",
            alt = self.alt_name,
            arch = self.arch_name,
            uid = self.usage_id,
        ))
    }

    fn kind_label(&self) -> &'static str {
        "room"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
    use std::collections::HashSet;
    use std::fs;

    use super::super::areatree::AreatreeIndex;
    use super::*;

    fn snapshot_with(areatree: &str) -> RepoSnapshot {
        RepoSnapshot {
            areatree: AreatreeIndex::parse(areatree).unwrap(),
            tumonline_room_codes: HashSet::from(["0101.01.999".to_string()]),
            user_added_room_codes: HashSet::new(),
            poi_keys: HashSet::new(),
            usage_ids: HashSet::from([12]),
            coord_ids: HashSet::new(),
        }
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
            coords: None,
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

    #[test]
    fn validate_happy_path() {
        let r = sample_room();
        let s = snapshot_with(ar());
        r.validate("5117.EG.103", &s).unwrap();
    }

    #[test]
    fn validate_bad_room_code_segments() {
        let r = sample_room();
        let s = snapshot_with(ar());
        let err = r.validate("5117.EG", &s).unwrap_err();
        assert!(matches!(err, AdditionError::BadRoomCode(_, _)));
    }

    #[test]
    fn validate_prefix_mismatch() {
        let r = sample_room();
        let s = snapshot_with(ar());
        let err = r.validate("0101.EG.103", &s).unwrap_err();
        assert!(matches!(err, AdditionError::PrefixMismatch { .. }));
    }

    #[test]
    fn validate_unknown_parent() {
        let mut r = sample_room();
        r.parent_building_id = "9999".to_string();
        let s = snapshot_with(ar());
        let err = r.validate("9999.EG.103", &s).unwrap_err();
        assert!(matches!(err, AdditionError::UnknownParent { .. }));
    }

    #[test]
    fn validate_wrong_parent_type() {
        // Pointing parent_building_id at an area (id "m") instead of a building/joined_building.
        let mut r = sample_room();
        r.parent_building_id = "m".to_string();
        let s = snapshot_with(ar());
        let err = r.validate("m.EG.103", &s).unwrap_err();
        assert!(
            matches!(err, AdditionError::WrongParentType { .. }),
            "expected WrongParentType, got: {err}"
        );
    }

    #[test]
    fn validate_unknown_usage_id() {
        let mut r = sample_room();
        r.usage_id = 999;
        let s = snapshot_with(ar());
        let err = r.validate("5117.EG.103", &s).unwrap_err();
        assert!(matches!(err, AdditionError::UnknownUsageId(999)));
    }

    #[test]
    fn validate_bad_arch_name() {
        let mut r = sample_room();
        r.arch_name = "bad-arch-name".to_string();
        let s = snapshot_with(ar());
        let err = r.validate("5117.EG.103", &s).unwrap_err();
        assert!(matches!(err, AdditionError::BadArchName(_)));
    }

    #[test]
    fn validate_id_collision_with_tumonline() {
        let r = sample_room();
        let mut s = snapshot_with(ar());
        s.tumonline_room_codes.insert("5117.EG.103".to_string());
        let err = r.validate("5117.EG.103", &s).unwrap_err();
        assert!(matches!(err, AdditionError::IdCollision(_, _)));
    }

    #[test]
    fn apply_writes_yaml_additions() {
        let r = sample_room();
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources).unwrap();
        fs::write(
            sources.join("15_patches-rooms_tumonline.yaml"),
            "patches: []\n",
        )
        .unwrap();

        let summary = r.apply("5117.EG.103", dir.path(), "branch").unwrap();
        assert!(summary.contains("5117.EG.103"));
        let written = fs::read_to_string(sources.join("15_patches-rooms_tumonline.yaml")).unwrap();
        assert!(written.contains("additions:"));
        assert!(written.contains("room_key: 5117.EG.103"));
        assert!(written.contains("EG103@5117"));
    }

    #[test]
    fn apply_writes_coordinates_when_provided() {
        let mut r = sample_room();
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources).unwrap();
        fs::write(
            sources.join("15_patches-rooms_tumonline.yaml"),
            "patches: []\n",
        )
        .unwrap();
        fs::write(sources.join("coordinates.csv"), "id,lat,lon\n").unwrap();
        let coord_json = serde_json::json!({"lat": 48.262, "lon": 11.668});
        r.coords = Some(serde_json::from_value(coord_json).unwrap());

        r.apply("5117.EG.103", dir.path(), "branch").unwrap();
        let coords = fs::read_to_string(sources.join("coordinates.csv")).unwrap();
        assert!(coords.contains("5117.EG.103"));
        assert!(coords.contains("48.262"));
    }
}
