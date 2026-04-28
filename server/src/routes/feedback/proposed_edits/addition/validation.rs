//! Snapshot of the on-disk reference data + `AdditionError` shared by all addition variants.
//!
//! The validation runs against the SAME files an addition will eventually modify, after the
//! `TempRepo` has been cloned (and after any prior edits applied in the same batch). This
//! avoids the in-memory cache staleness problem.
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead as _, BufReader};
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use super::areatree::AreatreeIndex;

#[derive(Debug, Error)]
pub enum AdditionError {
    #[error("ID `{0}` already exists in {1}")]
    IdCollision(String, &'static str),
    #[error("parent `{parent}` does not exist")]
    UnknownParent { parent: String },
    #[error("parent `{parent}` has type `{actual}`, but {kind} requires one of {expected:?}")]
    WrongParentType {
        parent: String,
        actual: String,
        kind: &'static str,
        expected: &'static [&'static str],
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
    #[error("coords required for {0}")]
    MissingCoords(&'static str),
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

/// Cached on-disk reference data that all addition validators consult.
#[derive(Debug)]
pub struct RepoSnapshot {
    pub areatree: AreatreeIndex,
    pub tumonline_room_codes: HashSet<String>,
    pub user_added_room_codes: HashSet<String>,
    pub poi_keys: HashSet<String>,
    pub usage_ids: HashSet<u32>,
    /// Existing IDs in `coordinates.csv`. Reserved for future cross-checks (e.g. warning when
    /// a new building has the same coords as an existing one) — currently only loaded.
    #[allow(dead_code)]
    pub coord_ids: HashSet<String>,
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
        let areatree_content = fs::read_to_string(&areatree_path)?;
        let areatree = AreatreeIndex::parse(&areatree_content)?;

        let pois_path = base_dir.join("data").join("sources").join("21_pois.yaml");
        let poi_keys = if pois_path.exists() {
            let s = fs::read_to_string(&pois_path)?;
            let map: BTreeMap<String, serde_yaml::Value> =
                serde_yaml::from_str(&s).unwrap_or_default();
            map.into_keys().collect()
        } else {
            HashSet::new()
        };

        let patches_path = base_dir
            .join("data")
            .join("sources")
            .join("15_patches-rooms_tumonline.yaml");
        let user_added_room_codes = if patches_path.exists() {
            let s = fs::read_to_string(&patches_path)?;
            let parsed: PatchesFile = serde_yaml::from_str(&s).unwrap_or_default();
            parsed
                .additions
                .into_iter()
                .map(|a| a.room_key)
                .filter(|k| !k.is_empty())
                .collect()
        } else {
            HashSet::new()
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

        let coords_csv = base_dir
            .join("data")
            .join("sources")
            .join("coordinates.csv");
        let coord_ids = read_first_column(&coords_csv).unwrap_or_default();

        Ok(Self {
            areatree,
            tumonline_room_codes,
            user_added_room_codes,
            poi_keys,
            usage_ids,
            coord_ids,
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
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
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
            sources.join("coordinates.csv"),
            "id,lat,lon\n0101.01.101,1.0,1.0\n",
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
        assert!(snap.coord_ids.contains("0101.01.101"));
        assert!(snap.poi_keys.contains("validierungsautomat-1"));
        assert!(snap.user_added_room_codes.contains("5117.EG.999"));
    }
}
