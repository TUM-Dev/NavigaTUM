//! Strict, typed additions for new rooms, buildings, and POIs.
//!
//! The dispatcher is the [`Addition`] enum (a `serde`-tagged sum); each variant has its own
//! writer module and shares a validation step that consults a [`validation::RepoSnapshot`] of
//! the on-disk reference data.
//!
//! The flow per request entry:
//!   1. The handler builds a [`validation::RepoSnapshot`] from the cloned `TempRepo`.
//!   2. Each addition is validated against the snapshot — any failure surfaces as 422.
//!   3. After all additions validate, each one calls [`AppliableAddition::apply`] which writes
//!      to the right files and returns a human-readable summary string for the PR description.
use std::path::Path;

use serde::Deserialize;

pub mod areatree;
pub mod building;
pub mod poi;
pub mod room;
pub mod validation;

use building::NewBuilding;
use poi::NewPoi;
use room::NewRoom;
use validation::{AdditionError, RepoSnapshot};

#[derive(Debug, Deserialize, Clone, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Addition {
    Room(NewRoom),
    Building(NewBuilding),
    Poi(NewPoi),
}

pub trait AppliableAddition {
    fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError>;
    fn apply(&self, key: &str, base_dir: &Path, branch: &str) -> anyhow::Result<String>;
    /// Stable label for the variant — used by the description renderer and PR title.
    fn kind_label(&self) -> &'static str;
}

impl Addition {
    fn as_appliable(&self) -> &dyn AppliableAddition {
        match self {
            Self::Room(r) => r,
            Self::Building(b) => b,
            Self::Poi(p) => p,
        }
    }

    pub fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        self.as_appliable().validate(key, snap)
    }

    pub fn apply(&self, key: &str, base_dir: &Path, branch: &str) -> anyhow::Result<String> {
        self.as_appliable().apply(key, base_dir, branch)
    }

    pub fn kind_label(&self) -> &'static str {
        self.as_appliable().kind_label()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::panic_in_result_fn)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_room_variant() {
        let json = serde_json::json!({
            "kind": "room",
            "parent_building_id": "5117",
            "alt_name": "Testraum",
            "arch_name": "EG103@5117",
            "usage_id": 12,
            "coords": {"lat": 48.262, "lon": 11.668}
        });
        let a: Addition = serde_json::from_value(json).unwrap();
        assert_eq!(a.kind_label(), "room");
    }

    #[test]
    fn deserializes_building_variant() {
        // The inner enum `kind` field is renamed to `node_kind` to avoid colliding with the
        // outer tag (`kind`).
        let json = serde_json::json!({
            "kind": "building",
            "parent_id": "stammgelaende",
            "node_kind": "building",
            "building_prefixes": ["5117"],
            "name": "Foo",
            "coords": {"lat": 48.0, "lon": 11.0}
        });
        let a: Addition = serde_json::from_value(json).unwrap();
        assert_eq!(a.kind_label(), "building");
    }
}
