//! Validation runs against [`validation::RepoSnapshot`] before any writes so a malformed
//! addition surfaces as 422 rather than landing as a broken PR.
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
        // `node_kind` instead of `kind` to avoid colliding with the outer serde tag.
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
