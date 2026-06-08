use std::path::{Path, PathBuf};

use anyhow::ensure;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use url::Url;

use super::AppliableEdit;
use super::csv_edit::apply_csv_upsert;

/// A correction to an entry's opening hours.
///
/// Routes into the data pipeline's canonical `opening_hours.csv` (one schedule
/// per entry id) rather than a runtime override, so an accepted fix flows back
/// to source. The client's day/time builder assembles `opening_hours` into a
/// plain OSM `opening_hours` string; `last_update` is stamped server-side so the
/// provenance date can't be spoofed, and the optional `valid_from`/`valid_until`/
/// `service` columns are left to maintainers (preserved on update, empty on
/// insert).
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct OpeningHoursEdit {
    /// OSM `opening_hours` string assembled by the client's day/time builder.
    #[schema(example = "Mo-Fr 08:00-20:00; Sa 09:00-14:00")]
    opening_hours: String,
    /// Absolute http(s) URL documenting the schedule (e.g. the official department page).
    #[schema(example = "https://www.ub.tum.de/oeffnungszeiten")]
    source_url: Url,
}

impl OpeningHoursEdit {
    fn opening_hours_csv_path(base_dir: &Path) -> PathBuf {
        base_dir
            .join("data")
            .join("sources")
            .join("opening_hours.csv")
    }

    /// Split out so tests can pin `last_update` instead of reading the wall clock.
    fn apply_with_date(&self, key: &str, base_dir: &Path, last_update: &str) -> anyhow::Result<()> {
        let opening_hours = self.opening_hours.trim();
        ensure!(!opening_hours.is_empty(), "opening_hours must not be empty");
        ensure!(
            matches!(self.source_url.scheme(), "http" | "https"),
            "source_url must be an http(s) URL, got {:?}",
            self.source_url.scheme()
        );

        // Leading columns the edit owns; trailing valid_from/valid_until/service
        // are padded empty on insert and preserved on update by `apply_csv_upsert`.
        apply_csv_upsert(
            key,
            &Self::opening_hours_csv_path(base_dir),
            &[
                key.to_string(),
                opening_hours.to_string(),
                self.source_url.to_string(),
                last_update.to_string(),
            ],
        )
    }
}

impl AppliableEdit for OpeningHoursEdit {
    fn apply(&self, key: &str, base_dir: &Path, _branch: &str) -> anyhow::Result<String> {
        let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
        self.apply_with_date(key, base_dir, &today)?;
        Ok(format!(
            "`{}` ([source]({}))",
            self.opening_hours.trim(),
            self.source_url
        ))
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
    use std::fs;

    use insta::assert_snapshot;

    use super::*;

    const HEADER: &str = "id,opening_hours,source_url,last_update,valid_from,valid_until,service";

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().unwrap();
        let sources_dir = dir.path().join("data").join("sources");
        fs::create_dir_all(&sources_dir).unwrap();
        fs::write(sources_dir.join("opening_hours.csv"), format!("{HEADER}\n")).unwrap();
        (dir, sources_dir)
    }

    fn edit(opening_hours: &str, source_url: &str) -> OpeningHoursEdit {
        OpeningHoursEdit {
            opening_hours: opening_hours.to_string(),
            source_url: Url::parse(source_url).unwrap(),
        }
    }

    #[test]
    fn test_insert_pads_optional_columns() {
        let (dir, sources_dir) = setup();
        edit("Mo-Fr 08:00-20:00", "https://example.com/hours")
            .apply_with_date("5304.EG.001", dir.path(), "2026-06-08")
            .unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("opening_hours.csv")).unwrap(), @r"
        id,opening_hours,source_url,last_update,valid_from,valid_until,service
        5304.EG.001,Mo-Fr 08:00-20:00,https://example.com/hours,2026-06-08,,,
        ");
    }

    #[test]
    fn test_sorted_insert() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("opening_hours.csv"),
            format!("{HEADER}\nalpha,Mo off,https://a.com,2026-01-01,,,\nzulu,Su off,https://z.com,2026-01-01,,,\n"),
        )
        .unwrap();
        edit("Mo-Fr 09:00-17:00", "https://m.com/hours")
            .apply_with_date("mike", dir.path(), "2026-06-08")
            .unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("opening_hours.csv")).unwrap(), @r"
        id,opening_hours,source_url,last_update,valid_from,valid_until,service
        alpha,Mo off,https://a.com,2026-01-01,,,
        mike,Mo-Fr 09:00-17:00,https://m.com/hours,2026-06-08,,,
        zulu,Su off,https://z.com,2026-01-01,,,
        ");
    }

    #[test]
    fn test_update_preserves_curated_optional_columns() {
        let (dir, sources_dir) = setup();
        fs::write(
            sources_dir.join("opening_hours.csv"),
            format!("{HEADER}\nroom1,Mo 08:00-12:00,https://old.com,2025-01-01,2025-09-01,2026-03-01,Mensa\n"),
        )
        .unwrap();
        edit("Mo-Fr 08:00-20:00", "https://new.com/hours")
            .apply_with_date("room1", dir.path(), "2026-06-08")
            .unwrap();
        // opening_hours, source_url, last_update refreshed; validity range + service kept.
        assert_snapshot!(fs::read_to_string(sources_dir.join("opening_hours.csv")).unwrap(), @r"
        id,opening_hours,source_url,last_update,valid_from,valid_until,service
        room1,Mo-Fr 08:00-20:00,https://new.com/hours,2026-06-08,2025-09-01,2026-03-01,Mensa
        ");
    }

    #[test]
    fn test_csv_escapes_comma_separated_hours() {
        let (dir, sources_dir) = setup();
        edit("Mo-Fr 08:00-12:00,13:00-17:00", "https://example.com/hours")
            .apply_with_date("room1", dir.path(), "2026-06-08")
            .unwrap();
        assert_snapshot!(fs::read_to_string(sources_dir.join("opening_hours.csv")).unwrap(), @r#"
        id,opening_hours,source_url,last_update,valid_from,valid_until,service
        room1,"Mo-Fr 08:00-12:00,13:00-17:00",https://example.com/hours,2026-06-08,,,
        "#);
    }

    #[test]
    fn test_rejects_empty_opening_hours() {
        let (dir, _sources_dir) = setup();
        let err = edit("   ", "https://example.com")
            .apply_with_date("room1", dir.path(), "2026-06-08")
            .unwrap_err();
        assert!(err.to_string().contains("must not be empty"), "{err}");
    }

    #[test]
    fn test_apply_returns_sourced_description() {
        let (dir, _sources_dir) = setup();
        let desc = edit("Mo-Fr 08:00-20:00", "https://example.com/hours")
            .apply("room1", dir.path(), "branch")
            .unwrap();
        assert_eq!(
            desc,
            "`Mo-Fr 08:00-20:00` ([source](https://example.com/hours))"
        );
    }
}
