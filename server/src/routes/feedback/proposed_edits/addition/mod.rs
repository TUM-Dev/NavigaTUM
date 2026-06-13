//! Validation runs against [`validation::RepoSnapshot`] before any writes so a malformed
//! addition surfaces as 422 rather than landing as a broken PR.
use std::path::Path;

use serde::Deserialize;

pub mod areatree;
pub mod building;
pub mod event;
pub mod poi;
pub mod room;
pub mod validation;

use building::NewBuilding;
use event::NewEvent;
use poi::NewPoi;
use room::NewRoom;
use validation::{AdditionError, RepoSnapshot};

#[derive(Debug, Deserialize, Clone, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Addition {
    Room(NewRoom),
    Building(NewBuilding),
    Poi(NewPoi),
    Event(NewEvent),
}

pub trait AppliableAddition {
    fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError>;
    fn apply(&self, key: &str, base_dir: &Path, branch: &str) -> anyhow::Result<AppliedAddition>;
    fn kind_label(&self) -> &'static str;
}

/// The result of applying an [`Addition`], as the PR description renders it.
pub struct AppliedAddition {
    /// One-line human summary for the PR body.
    pub summary: String,
    /// Image to show beside the summary (events only).
    pub image_url: Option<String>,
    /// Whether an existing entry was replaced rather than created (events only).
    pub replaced: bool,
}

impl AppliedAddition {
    /// A new addition with no image.
    pub fn created(summary: String) -> Self {
        Self {
            summary,
            image_url: None,
            replaced: false,
        }
    }
}

impl Addition {
    fn as_appliable(&self) -> &dyn AppliableAddition {
        match self {
            Self::Room(r) => r,
            Self::Building(b) => b,
            Self::Poi(p) => p,
            Self::Event(e) => e,
        }
    }

    pub fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        self.as_appliable().validate(key, snap)
    }

    pub fn apply(
        &self,
        key: &str,
        base_dir: &Path,
        branch: &str,
    ) -> anyhow::Result<AppliedAddition> {
        self.as_appliable().apply(key, base_dir, branch)
    }

    pub fn kind_label(&self) -> &'static str {
        self.as_appliable().kind_label()
    }

    /// The event's name, or `None` for non-event additions.
    pub fn event_name(&self) -> Option<&str> {
        match self {
            Self::Event(e) => Some(&e.name),
            _ => None,
        }
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

    #[test]
    fn deserializes_event_variant() {
        let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M8AAAMBAQAY3Y2wAAAAAElFTkSuQmCC";
        let json = serde_json::json!({
            "kind": "event",
            "image": { "content": png, "metadata": { "author": "Studi", "license": { "text": "CC-BY" } } },
            "name": "GARNIX Festival",
            "description": "Open-air student festival.",
            "starts_at": "2026-06-10T16:00:00+02:00",
            "ends_at": "2026-06-12T23:00:00+02:00",
            "coords": {"lat": 48.262908, "lon": 11.669102},
            "organising_org_id": 51897
        });
        let a: Addition = serde_json::from_value(json).unwrap();
        assert_eq!(a.kind_label(), "event");
    }
}
