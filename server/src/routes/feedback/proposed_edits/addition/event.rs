use std::fs::{self, OpenOptions};
use std::path::Path;

use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::Deserialize;

use super::super::AppliableEdit as _;
use super::super::coordinate::Coordinate;
use super::super::image::Image;
use super::AppliableAddition;
use super::validation::{AdditionError, RepoSnapshot};

const MAX_NAME_LEN: usize = 200;
/// The photo marker renders a 256×256 thumb crop, so smaller uploads would be upscaled.
const MIN_IMAGE_DIM: u32 = 256;
const MAX_HORIZON_DAYS: i64 = 365;
const MAX_DURATION_DAYS: i64 = 30;

/// A proposed campus event, appended as a row to `data/sources/events.csv`.
#[derive(Debug, Deserialize, Clone, utoipa::ToSchema)]
pub struct NewEvent {
    pub image: Image,
    pub name: String,
    pub description: String,
    /// RFC3339 string, to match the CSV/parquet contract.
    pub starts_at: String,
    pub ends_at: String,
    pub coords: Coordinate,
    pub organising_org_id: i32,
}

/// A sha256 hex digest is at most 64 chars.
const MAX_EVENT_HASH_LEN: usize = 64;
fn is_valid_event_key(key: &str) -> bool {
    let Some(hash) = key.strip_prefix("event_") else {
        return false;
    };
    !hash.is_empty()
        && hash.len() <= MAX_EVENT_HASH_LEN
        && hash
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// `D.M.YY HH:MM` (no zero padding) in the event's own offset.
fn format_de(value: &str) -> anyhow::Result<String> {
    Ok(DateTime::parse_from_rfc3339(value)?
        .format("%-d.%-m.%y %H:%M")
        .to_string())
}

fn parse_rfc3339(field: &'static str, value: &str) -> Result<DateTime<FixedOffset>, AdditionError> {
    #[expect(
        clippy::map_err_ignore,
        reason = "the field name and offending value fully describe the 422; chrono's parse detail is not user-actionable"
    )]
    DateTime::parse_from_rfc3339(value).map_err(|_| AdditionError::BadTimestamp {
        field,
        value: value.to_string(),
    })
}

impl NewEvent {
    fn validate_temporal(
        &self,
        starts_at: DateTime<FixedOffset>,
        ends_at: DateTime<FixedOffset>,
        now: DateTime<Utc>,
    ) -> Result<(), AdditionError> {
        let starts_at = starts_at.with_timezone(&Utc);
        let ends_at = ends_at.with_timezone(&Utc);
        // Reversed ranges pass the checks below and would only fail in EventsSchema at build
        // time, breaking the shared batch PR.
        if ends_at < starts_at {
            return Err(AdditionError::EventEndsBeforeStart {
                starts_at: self.starts_at.clone(),
                ends_at: self.ends_at.clone(),
            });
        }
        if ends_at <= now {
            return Err(AdditionError::EventEnded {
                ends_at: self.ends_at.clone(),
                now: now.to_rfc3339(),
            });
        }
        if starts_at > now + Duration::days(MAX_HORIZON_DAYS) {
            return Err(AdditionError::EventStartTooFarOut {
                starts_at: self.starts_at.clone(),
                max_days: MAX_HORIZON_DAYS,
            });
        }
        if ends_at - starts_at > Duration::days(MAX_DURATION_DAYS) {
            return Err(AdditionError::EventTooLong {
                max_days: MAX_DURATION_DAYS,
            });
        }
        Ok(())
    }

    fn validate_image(&self) -> Result<(), AdditionError> {
        let (width, height) = self
            .image
            .decoded_dimensions()
            .map_err(|e| AdditionError::BadImage(e.to_string()))?;
        if width.min(height) < MIN_IMAGE_DIM {
            return Err(AdditionError::ImageTooSmall {
                width,
                height,
                min: MIN_IMAGE_DIM,
            });
        }
        Ok(())
    }

    fn append_events_row(&self, image: &str, base_dir: &Path) -> anyhow::Result<()> {
        let csv_path = base_dir.join("data").join("sources").join("events.csv");
        // Guard a missing trailing newline, so the row isn't glued onto the last one.
        if let Some(&last) = fs::read(&csv_path)?.last()
            && last != b'\n'
        {
            anyhow::bail!("events.csv does not end with a newline; refusing to append");
        }
        let file = OpenOptions::new().append(true).open(&csv_path)?;
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        writer.write_record([
            image,
            &self.coords.lat().to_string(),
            &self.coords.lon().to_string(),
            &self.name,
            &self.starts_at,
            &self.ends_at,
            &self.description,
            &self.organising_org_id.to_string(),
            self.image.author(),
        ])?;
        writer.flush()?;
        Ok(())
    }
}

impl AppliableAddition for NewEvent {
    fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        if !is_valid_event_key(key) {
            return Err(AdditionError::BadId(key.to_string()));
        }
        if self.name.trim().is_empty() || self.name.len() > MAX_NAME_LEN {
            return Err(AdditionError::BadName {
                len: self.name.len(),
                max: MAX_NAME_LEN,
            });
        }
        if self.description.trim().is_empty() {
            return Err(AdditionError::BadDescription);
        }
        let starts_at = parse_rfc3339("starts_at", &self.starts_at)?;
        let ends_at = parse_rfc3339("ends_at", &self.ends_at)?;
        self.validate_temporal(starts_at, ends_at, snap.now)?;
        if !snap.org_ids.contains(&self.organising_org_id) {
            return Err(AdditionError::UnknownOrgId(self.organising_org_id));
        }
        self.validate_image()?;
        Ok(())
    }

    fn apply(&self, key: &str, base_dir: &Path, branch: &str) -> anyhow::Result<String> {
        let image_md = self.image.apply(key, base_dir, branch)?;
        // Content-addressed key, so always the first slot.
        let image = format!("/cdn/thumb/{key}_0.webp");
        self.append_events_row(&image, base_dir)?;

        Ok(format!(
            "new event `{name}` ({starts_at} - {ends_at}, org `{org}`)\n\n{image_md}",
            name = self.name,
            starts_at = format_de(&self.starts_at)?,
            ends_at = format_de(&self.ends_at)?,
            org = self.organising_org_id,
        ))
    }

    fn kind_label(&self) -> &'static str {
        "event"
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::indexing_slicing,
        clippy::needless_pass_by_value,
        reason = "tests assert via panic/unwrap and index into known-shape JSON fixtures"
    )]
    use std::collections::HashSet;
    use std::io::Cursor;

    use base64::Engine as _;
    use base64::prelude::BASE64_STANDARD;
    use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
    use insta::assert_snapshot;
    use rstest::rstest;
    use serde_json::json;

    use super::super::areatree::AreatreeIndex;
    use super::*;

    fn image_b64(width: u32, height: u32) -> String {
        let buf = ImageBuffer::from_fn(width, height, |_, _| Rgb([120u8, 120, 120]));
        let mut bytes = Vec::new();
        DynamicImage::ImageRgb8(buf)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        BASE64_STANDARD.encode(&bytes)
    }

    fn fixed_now() -> DateTime<Utc> {
        "2026-06-07T12:00:00Z".parse().unwrap()
    }

    fn snapshot_at(now: DateTime<Utc>) -> RepoSnapshot {
        RepoSnapshot {
            areatree: AreatreeIndex::parse(":Standorte:root[root]\n").unwrap(),
            tumonline_room_codes: HashSet::new(),
            user_added_room_codes: HashSet::new(),
            poi_keys: HashSet::new(),
            usage_ids: HashSet::new(),
            org_ids: HashSet::from([51897]),
            now,
        }
    }

    fn event_json(image: serde_json::Value) -> serde_json::Value {
        json!({
            "image": {
                "content": image,
                "metadata": { "author": "Studi", "license": { "text": "CC-BY" } }
            },
            "name": "GARNIX Festival",
            "description": "Live music, food trucks, and stands.",
            "starts_at": "2026-06-10T16:00:00+02:00",
            "ends_at": "2026-06-12T23:00:00+02:00",
            "coords": { "lat": 48.262908, "lon": 11.669102 },
            "organising_org_id": 51897
        })
    }

    fn sample_event() -> NewEvent {
        serde_json::from_value(event_json(json!(image_b64(300, 300)))).unwrap()
    }

    #[test]
    fn format_de_drops_leading_zeros_and_keeps_local_offset() {
        assert_eq!(
            format_de("2026-06-05T09:07:00+02:00").unwrap(),
            "5.6.26 09:07"
        );
    }

    #[test]
    fn deserializes_and_validates_happy() {
        sample_event()
            .validate("event_9d02ddd940c43f87", &snapshot_at(fixed_now()))
            .unwrap();
    }

    #[test]
    fn missing_image_fails_to_deserialize() {
        let mut value = event_json(json!(image_b64(300, 300)));
        value.as_object_mut().unwrap().remove("image");
        let err = serde_json::from_value::<NewEvent>(value).unwrap_err();
        assert!(err.to_string().contains("image"), "got: {err}");
    }

    type Check = fn(&AdditionError) -> bool;

    fn event_with(mutate: impl FnOnce(&mut serde_json::Value)) -> NewEvent {
        let mut v = event_json(json!(image_b64(300, 300)));
        mutate(&mut v);
        serde_json::from_value(v).unwrap()
    }

    #[rstest]
    #[case::bad_key("event_NOTHEX", fixed_now(), sample_event(), (|e| matches!(e, AdditionError::BadId(_))) as Check)]
    #[case::missing_prefix("9d02ddd940c43f87", fixed_now(), sample_event(), (|e| matches!(e, AdditionError::BadId(_))) as Check)]
    #[case::ended(
        "event_abc",
        "2027-01-01T00:00:00Z".parse().unwrap(),
        sample_event(),
        (|e| matches!(e, AdditionError::EventEnded { .. })) as Check
    )]
    #[case::too_far_out(
        "event_abc",
        "2024-01-01T00:00:00Z".parse().unwrap(),
        sample_event(),
        (|e| matches!(e, AdditionError::EventStartTooFarOut { .. })) as Check
    )]
    #[case::too_long(
        "event_abc",
        fixed_now(),
        event_with(|v| v["ends_at"] = json!("2026-08-10T23:00:00+02:00")),
        (|e| matches!(e, AdditionError::EventTooLong { .. })) as Check
    )]
    #[case::ends_before_start(
        "event_abc",
        fixed_now(),
        event_with(|v| {
            v["starts_at"] = json!("2026-06-12T16:00:00+02:00");
            v["ends_at"] = json!("2026-06-10T23:00:00+02:00");
        }),
        (|e| matches!(e, AdditionError::EventEndsBeforeStart { .. })) as Check
    )]
    #[case::bad_timestamp(
        "event_abc",
        fixed_now(),
        event_with(|v| v["starts_at"] = json!("not-a-date")),
        (|e| matches!(e, AdditionError::BadTimestamp { .. })) as Check
    )]
    #[case::empty_name(
        "event_abc",
        fixed_now(),
        event_with(|v| v["name"] = json!("  ")),
        (|e| matches!(e, AdditionError::BadName { .. })) as Check
    )]
    #[case::empty_description(
        "event_abc",
        fixed_now(),
        event_with(|v| v["description"] = json!("")),
        (|e| matches!(e, AdditionError::BadDescription)) as Check
    )]
    #[case::image_too_small(
        "event_abc",
        fixed_now(),
        serde_json::from_value(event_json(json!(image_b64(100, 100)))).unwrap(),
        (|e| matches!(e, AdditionError::ImageTooSmall { .. })) as Check
    )]
    #[case::bad_image(
        "event_abc",
        fixed_now(),
        serde_json::from_value(event_json(json!("not-base64-image!!"))).unwrap(),
        (|e| matches!(e, AdditionError::BadImage(_))) as Check
    )]
    fn validate_failure_cases(
        #[case] key: &str,
        #[case] now: DateTime<Utc>,
        #[case] event: NewEvent,
        #[case] check: Check,
    ) {
        let err = event.validate(key, &snapshot_at(now)).unwrap_err();
        assert!(check(&err), "got: {err}");
    }

    #[test]
    fn unknown_org_id_is_rejected() {
        let mut value = event_json(json!(image_b64(300, 300)));
        value["organising_org_id"] = json!(999_999);
        let event: NewEvent = serde_json::from_value(value).unwrap();
        let err = event
            .validate("event_abc", &snapshot_at(fixed_now()))
            .unwrap_err();
        assert!(
            matches!(err, AdditionError::UnknownOrgId(999_999)),
            "got: {err}"
        );
    }

    #[test]
    fn apply_appends_row_and_saves_image() {
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        let lg = sources.join("img").join("lg");
        fs::create_dir_all(&lg).unwrap();
        fs::write(sources.join("img").join("img-sources.yaml"), "{}\n").unwrap();
        fs::write(
            sources.join("events.csv"),
            "event_image,event_lat,event_lon,event_name,event_datetime_start_at,event_datetime_end_at,event_description,event_organising_org_id,event_image_author\n",
        )
        .unwrap();

        let summary = sample_event()
            .apply("event_9d02ddd940c43f87", dir.path(), "branch")
            .unwrap();
        assert_snapshot!(summary, @r"
        new event `GARNIX Festival` (10.6.26 16:00 - 12.6.26 23:00, org `51897`)

        ![image showing event_9d02ddd940c43f87](https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/refs/heads/branch/data/sources/img/lg/event_9d02ddd940c43f87_0.webp)
        ");

        let csv = fs::read_to_string(sources.join("events.csv")).unwrap();
        assert_snapshot!(csv, @r#"
        event_image,event_lat,event_lon,event_name,event_datetime_start_at,event_datetime_end_at,event_description,event_organising_org_id,event_image_author
        /cdn/thumb/event_9d02ddd940c43f87_0.webp,48.262908,11.669102,GARNIX Festival,2026-06-10T16:00:00+02:00,2026-06-12T23:00:00+02:00,"Live music, food trucks, and stands.",51897,Studi
        "#);

        let yaml = fs::read_to_string(sources.join("img").join("img-sources.yaml")).unwrap();
        assert_snapshot!(yaml, @r"
        event_9d02ddd940c43f87:
          0:
            author: Studi
            license:
              text: CC-BY
        ");
        assert!(lg.join("event_9d02ddd940c43f87_0.webp").exists());
    }
}
