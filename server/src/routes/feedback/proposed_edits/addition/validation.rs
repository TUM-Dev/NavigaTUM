//! Validates against the cloned `TempRepo` itself, not an in-memory cache, so additions can't
//! pass validation against state that has already drifted from disk.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs::{self, File};
use std::io::{BufRead as _, BufReader, ErrorKind};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use thiserror::Error;

use super::areatree::{AreatreeIndex, AreatreeKind};
use super::event::event_key_of_image_path;

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
    Event,
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
    #[error("description must be non-empty")]
    BadDescription,
    #[error("`{field}` is not a valid RFC3339 timestamp: {value}")]
    BadTimestamp { field: &'static str, value: String },
    #[error("event ends_at {ends_at} is before starts_at {starts_at}")]
    EventEndsBeforeStart { starts_at: String, ends_at: String },
    #[error("event already ended (ends_at {ends_at} is not after now {now})")]
    EventEnded { ends_at: String, now: String },
    #[error("event starts more than {max_days} days out (starts_at {starts_at})")]
    EventStartTooFarOut { starts_at: String, max_days: i64 },
    #[error("event lasts longer than {max_days} days")]
    EventTooLong { max_days: i64 },
    #[error("image could not be decoded: {0}")]
    BadImage(String),
    #[error("image is {width}x{height}px, below the {min}px minimum on the shorter edge")]
    ImageTooSmall { width: u32, height: u32, min: u32 },
    #[error("organising_org_id {0} is not in orgs-de_tumonline.csv")]
    UnknownOrgId(i32),
    #[error("events.csv contains {count} rows for `{key}`; cannot determine which row to replace")]
    DuplicateEventRows { key: String, count: usize },
}

#[derive(Debug)]
pub struct RepoSnapshot {
    pub areatree: AreatreeIndex,
    pub tumonline_room_codes: HashSet<String>,
    pub user_added_room_codes: HashSet<String>,
    pub poi_keys: HashSet<String>,
    pub usage_ids: HashSet<u32>,
    pub org_ids: HashSet<i32>,
    /// How many `events.csv` rows derive from each event key; an event addition under a key
    /// with exactly one row is an update, more than one is a validation failure.
    pub event_row_counts: HashMap<String, usize>,
    /// Request time, so now-relative addition rules are deterministic per request.
    pub now: DateTime<Utc>,
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

        let orgs_csv = base_dir
            .join("data")
            .join("external")
            .join("results")
            .join("orgs-de_tumonline.csv");
        let org_ids = read_first_column(&orgs_csv)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        let events_csv = base_dir.join("data").join("sources").join("events.csv");
        let event_row_counts = match File::open(&events_csv) {
            Ok(file) => {
                let mut counts: HashMap<String, usize> = HashMap::new();
                // Descriptions span multiple lines, so this needs a real CSV parser.
                for record in csv::Reader::from_reader(file).into_records() {
                    let record = record?;
                    if let Some(key) = record.get(0).and_then(event_key_of_image_path) {
                        *counts.entry(key.to_string()).or_default() += 1;
                    }
                }
                counts
            }
            Err(e) if e.kind() == ErrorKind::NotFound => HashMap::new(),
            Err(e) => return Err(e.into()),
        };

        Ok(Self {
            areatree,
            tumonline_room_codes,
            user_added_room_codes,
            poi_keys,
            usage_ids,
            org_ids,
            event_row_counts,
            now: Utc::now(),
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
            ext.join("orgs-de_tumonline.csv"),
            "org_id,code,name,path\n1,TU00000,Technische Universität München,TUM\n51897,SV,Studentische Vertretung,TUM/SV\n",
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
        assert!(snap.org_ids.contains(&51897));
        assert!(snap.org_ids.contains(&1));
        assert!(snap.poi_keys.contains("validierungsautomat-1"));
        assert!(snap.user_added_room_codes.contains("5117.EG.999"));
        // No events.csv in this fixture, so no event rows are counted.
        assert!(snap.event_row_counts.is_empty());
    }

    #[test]
    fn counts_event_rows_by_key() {
        let dir = build_dir();
        // The duplicate `event_aaa` rows and the multi-line quoted description are the two
        // legacy-data shapes the counting has to survive.
        fs::write(
            dir.path().join("data").join("sources").join("events.csv"),
            "event_image,event_lat,event_lon,event_name,event_datetime_start_at,event_datetime_end_at,event_description,event_organising_org_id,event_image_author\n\
            /cdn/thumb/event_aaa_0.webp,48.1,11.5,A,2026-06-15T16:00:00+02:00,2026-06-16T16:00:00+02:00,\"multi\nline, description\",1,Studi\n\
            /cdn/thumb/event_aaa_1.webp,48.1,11.5,A again,2026-06-15T16:00:00+02:00,2026-06-16T16:00:00+02:00,duplicate key,1,Studi\n\
            /cdn/thumb/event_bbb_0.webp,48.2,11.6,B,2026-06-15T16:00:00+02:00,2026-06-16T16:00:00+02:00,single,1,Studi\n",
        )
        .unwrap();
        let snap = RepoSnapshot::load(dir.path()).unwrap();
        assert_eq!(snap.event_row_counts.get("event_aaa"), Some(&2));
        assert_eq!(snap.event_row_counts.get("event_bbb"), Some(&1));
        assert_eq!(snap.event_row_counts.len(), 2);
    }
}
