//! Validates against the cloned `TempRepo` itself, not an in-memory cache, so additions can't
//! pass validation against state that has already drifted from disk.
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs::{self, File};
use std::io::{BufRead as _, BufReader, ErrorKind};
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use super::areatree::{AreatreeIndex, AreatreeKind};

#[derive(Debug, Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum CollisionSource {
    #[strum(serialize = "rooms_tumonline.csv")]
    TumonlineRooms,
    #[strum(serialize = "15_patches-rooms_tumonline.yaml additions")]
    UserRoomAdditions,
    #[strum(serialize = "21_pois.yaml")]
    Pois,
    #[strum(serialize = "config.areatree")]
    Areatree,
}

impl fmt::Display for CollisionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).into())
    }
}

#[derive(Debug, Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum AdditionVariant {
    Room,
    Building,
    Poi,
}

impl fmt::Display for AdditionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).into())
    }
}

#[derive(Debug, Error)]
pub enum AdditionError {
    // `source` is reserved by thiserror (auto-wires `Error::source`); use `at` to avoid collision.
    #[error("ID `{id}` already exists in {at}")]
    IdCollision { id: String, at: CollisionSource },
    #[error("parent `{parent}` does not exist")]
    UnknownParent { parent: String },
    #[error("parent `{parent}` has type `{actual}`, but {kind} requires one of {expected:?}")]
    WrongParentType {
        parent: String,
        actual: AreatreeKind,
        kind: AdditionVariant,
        expected: &'static [AreatreeKind],
    },
    #[error("usage_id {0} is not in usages_tumonline.csv")]
    UnknownUsageId(u32),
    #[error("room code `{0}` is malformed: {1}")]
    BadRoomCode(String, &'static str),
    #[error(
        "room code `{code}` building prefix `{got}` does not match parent_building_id `{want}`"
    )]
    PrefixMismatch {
        code: String,
        got: String,
        want: String,
    },
    #[error("arch_name `{0}` does not match NAME@BUILDING with 4-digit building")]
    BadArchName(String),
    #[error("ID `{0}` does not match required slug shape (alphanum/dot/dash/underscore)")]
    BadId(String),
    #[error("name must be non-empty and ≤ {max} chars (got {len})")]
    BadName { len: usize, max: usize },
    #[error("usage_name must be non-empty")]
    BadUsageName,
    #[error("expected exactly one building_prefix for kind=Building, got {0}")]
    BuildingNeedsExactlyOnePrefix(usize),
    #[error("expected ≥ 2 building_prefixes for kind=JoinedBuilding, got {0}")]
    JoinedBuildingNeedsMultiplePrefixes(usize),
    #[error("building_prefix `{0}` must be 4 digits")]
    BadBuildingPrefix(String),
    #[error("building_prefix `{0}` already used elsewhere in areatree")]
    BuildingPrefixCollision(String),
    #[error("visible_id `{0}` already used elsewhere in areatree")]
    VisibleIdCollision(String),
}

#[derive(Debug)]
pub struct RepoSnapshot {
    pub areatree: AreatreeIndex,
    pub tumonline_room_codes: HashSet<String>,
    pub user_added_room_codes: HashSet<String>,
    pub poi_keys: HashSet<String>,
    pub usage_ids: HashSet<u32>,
}

#[derive(Debug, Deserialize, Default)]
struct PatchesFile {
    #[serde(default)]
    additions: Vec<RoomAdditionRaw>,
}

#[derive(Debug, Deserialize, Default)]
struct RoomAdditionRaw {
    #[serde(default)]
    room_key: String,
}

impl RepoSnapshot {
    pub fn load(base_dir: &Path) -> anyhow::Result<Self> {
        let areatree_path = base_dir
            .join("data")
            .join("processors")
            .join("areatree")
            .join("config.areatree");
        let areatree = AreatreeIndex::parse(&fs::read_to_string(&areatree_path)?)?;

        let pois_path = base_dir.join("data").join("sources").join("21_pois.yaml");
        let poi_keys = match fs::read_to_string(&pois_path) {
            Ok(s) => {
                let map: BTreeMap<String, serde_yaml::Value> =
                    serde_yaml::from_str(&s).unwrap_or_default();
                map.into_keys().collect()
            }
            Err(e) if e.kind() == ErrorKind::NotFound => HashSet::new(),
            Err(e) => return Err(e.into()),
        };

        let patches_path = base_dir
            .join("data")
            .join("sources")
            .join("15_patches-rooms_tumonline.yaml");
        let user_added_room_codes = match fs::read_to_string(&patches_path) {
            Ok(s) => {
                let parsed: PatchesFile = serde_yaml::from_str(&s).unwrap_or_default();
                parsed
                    .additions
                    .into_iter()
                    .map(|a| a.room_key)
                    .filter(|k| !k.is_empty())
                    .collect()
            }
            Err(e) if e.kind() == ErrorKind::NotFound => HashSet::new(),
            Err(e) => return Err(e.into()),
        };

        let rooms_csv = base_dir
            .join("data")
            .join("external")
            .join("results")
            .join("rooms_tumonline.csv");
        let tumonline_room_codes = read_first_column(&rooms_csv).unwrap_or_default();

        let usages_csv = base_dir
            .join("data")
            .join("external")
            .join("results")
            .join("usages_tumonline.csv");
        let usage_ids = read_first_column(&usages_csv)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();

        Ok(Self {
            areatree,
            tumonline_room_codes,
            user_added_room_codes,
            poi_keys,
            usage_ids,
        })
    }
}

fn read_first_column(path: &Path) -> anyhow::Result<HashSet<String>> {
    let f = File::open(path)?;
    let mut out = HashSet::new();
    for line in BufReader::new(f).lines().skip(1).map_while(Result::ok) {
        if let Some(first) = line.split(',').next() {
            let trimmed = first.trim();
            if !trimmed.is_empty() {
                out.insert(trimmed.to_string());
            }
        }
    }
    Ok(out)
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

    use super::*;

    fn build_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources).unwrap();
        let ext = dir.path().join("data").join("external").join("results");
        fs::create_dir_all(&ext).unwrap();
        let proc = dir.path().join("data").join("processors").join("areatree");
        fs::create_dir_all(&proc).unwrap();
        fs::write(
            proc.join("config.areatree"),
            ":Standorte:root[root]\n  0:Stammgelände:stammgelaende[campus]\n    01:Nordgelände:nordgelaende\n      0101:N1:0101,n1\n",
        )
        .unwrap();
        fs::write(
            ext.join("rooms_tumonline.csv"),
            "room_key,address_place\n0101.01.101,Munich\n0101.01.101A,Munich\n",
        )
        .unwrap();
        fs::write(
            ext.join("usages_tumonline.csv"),
            "usage_id,din277_id\n1,TF8.4\n12,NF\n",
        )
        .unwrap();
        fs::write(
            sources.join("21_pois.yaml"),
            "validierungsautomat-1:\n  parent: \"5101.EG.917\"\n  name: \"V1\"\n  usage: { name: V }\n",
        )
        .unwrap();
        fs::write(
            sources.join("15_patches-rooms_tumonline.yaml"),
            "patches: []\nadditions:\n  - room_key: 5117.EG.999\n    parent_building_id: 5117\n    alt_name: Existing user-added room\n    arch_name: EG999@5117\n    usage_id: 12\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn loads_all_indexes() {
        let dir = build_dir();
        let snap = RepoSnapshot::load(dir.path()).unwrap();
        assert!(snap.areatree.contains_id("0101"));
        assert!(snap.tumonline_room_codes.contains("0101.01.101"));
        assert!(snap.tumonline_room_codes.contains("0101.01.101A"));
        assert!(snap.usage_ids.contains(&12));
        assert!(snap.usage_ids.contains(&1));
        assert!(snap.poi_keys.contains("validierungsautomat-1"));
        assert!(snap.user_added_room_codes.contains("5117.EG.999"));
    }
}
