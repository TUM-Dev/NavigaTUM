use std::fs::{self, OpenOptions};
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str;

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

/// A proposed campus event, upserted by key into `data/sources/events.csv`: a row whose
/// image path derives from the addition key is replaced, otherwise the event is appended.
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

/// The addition key an `event_image` CSV value derives from (`/cdn/thumb/{key}_{slot}.webp`).
pub(super) fn event_key_of_image_path(image: &str) -> Option<&str> {
    let stem = image.strip_prefix("/cdn/thumb/")?.strip_suffix(".webp")?;
    let (key, slot) = stem.rsplit_once('_')?;
    (!slot.is_empty() && slot.bytes().all(|b| b.is_ascii_digit()) && is_valid_event_key(key))
        .then_some(key)
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

    fn write_row(&self, image: &str, out: impl io::Write) -> anyhow::Result<()> {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(out);
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

    fn append_events_row(&self, image: &str, csv_path: &Path) -> anyhow::Result<()> {
        // Guard a missing trailing newline, so the row isn't glued onto the last one.
        if let Some(&last) = fs::read(csv_path)?.last()
            && last != b'\n'
        {
            anyhow::bail!("events.csv does not end with a newline; refusing to append");
        }
        let file = OpenOptions::new().append(true).open(csv_path)?;
        self.write_row(image, file)
    }

    /// Splices the new row over `row` so every other byte of the CSV stays untouched.
    fn replace_events_row(
        &self,
        image: &str,
        csv_path: &Path,
        raw: &[u8],
        row: Range<usize>,
    ) -> anyhow::Result<()> {
        let before = raw
            .get(..row.start)
            .ok_or_else(|| anyhow::anyhow!("row start {} outside events.csv", row.start))?;
        let after = raw
            .get(row.end..)
            .ok_or_else(|| anyhow::anyhow!("row end {} outside events.csv", row.end))?;
        let mut out = Vec::with_capacity(raw.len());
        out.extend_from_slice(before);
        self.write_row(image, &mut out)?;
        out.extend_from_slice(after);
        fs::write(csv_path, out)?;
        Ok(())
    }
}

fn events_csv_path(base_dir: &Path) -> PathBuf {
    base_dir.join("data").join("sources").join("events.csv")
}

/// Byte range of the single `events.csv` row whose image path derives from `key`.
fn matching_row_range(raw: &[u8], key: &str) -> anyhow::Result<Option<Range<usize>>> {
    let mut reader = csv::Reader::from_reader(raw);
    reader.byte_headers()?;
    let mut record = csv::ByteRecord::new();
    let mut matched = None;
    loop {
        let start = usize::try_from(reader.position().byte())?;
        if !reader.read_byte_record(&mut record)? {
            break;
        }
        let image = str::from_utf8(record.get(0).unwrap_or_default())?;
        if event_key_of_image_path(image) == Some(key) {
            // validate() already rejects multi-match keys; a second hit here means the CSV
            // changed between validation and apply.
            anyhow::ensure!(
                matched.is_none(),
                "multiple events.csv rows match `{key}`; refusing to replace"
            );
            matched = Some(start..usize::try_from(reader.position().byte())?);
        }
    }
    Ok(matched)
}

impl AppliableAddition for NewEvent {
    fn validate(&self, key: &str, snap: &RepoSnapshot) -> Result<(), AdditionError> {
        if !is_valid_event_key(key) {
            return Err(AdditionError::BadId(key.to_string()));
        }
        // Identity = key, so one existing row is an update; more than one (legacy data
        // only) leaves no way to pick the row to replace.
        let row_count = snap.event_row_counts.get(key).copied().unwrap_or(0);
        if row_count > 1 {
            return Err(AdditionError::DuplicateEventRows {
                key: key.to_string(),
                count: row_count,
            });
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
        let csv_path = events_csv_path(base_dir);
        let raw = fs::read(&csv_path)?;
        // Content-addressed key, so always the first slot.
        let image = format!("/cdn/thumb/{key}_0.webp");
        // Identity = key: an existing row under this key makes the addition an update.
        let (verb, image_md) = if let Some(row) = matching_row_range(&raw, key)? {
            let image_md = self.image.replace(key, base_dir, branch)?;
            self.replace_events_row(&image, &csv_path, &raw, row)?;
            ("update", image_md)
        } else {
            let image_md = self.image.apply(key, base_dir, branch)?;
            self.append_events_row(&image, &csv_path)?;
            ("new", image_md)
        };

        Ok(format!(
            "{verb} event `{name}` ({starts_at} - {ends_at}, org `{org}`)\n\n{image_md}",
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
    use std::collections::{HashMap, HashSet};
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
            event_row_counts: HashMap::new(),
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

    #[rstest]
    #[case::slot0(
        "/cdn/thumb/event_9d02ddd940c43f87_0.webp",
        Some("event_9d02ddd940c43f87")
    )]
    #[case::higher_slot("/cdn/thumb/event_abc_12.webp", Some("event_abc"))]
    #[case::missing_leading_slash("cdn/thumb/event_abc_0.webp", None)]
    #[case::no_slot("/cdn/thumb/event_abc.webp", None)]
    #[case::non_numeric_slot("/cdn/thumb/event_abc_x.webp", None)]
    #[case::not_an_event("/cdn/thumb/5510.02.001_0.webp", None)]
    #[case::uppercase_hash("/cdn/thumb/event_ABC_0.webp", None)]
    fn derives_event_key_from_image_path(#[case] image: &str, #[case] expected: Option<&str>) {
        assert_eq!(event_key_of_image_path(image), expected);
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
    fn key_with_multiple_existing_rows_is_rejected() {
        let mut snap = snapshot_at(fixed_now());
        snap.event_row_counts
            .insert("event_9d02ddd940c43f87".to_string(), 2);
        let err = sample_event()
            .validate("event_9d02ddd940c43f87", &snap)
            .unwrap_err();
        assert!(
            matches!(err, AdditionError::DuplicateEventRows { count: 2, .. }),
            "got: {err}"
        );
    }

    #[test]
    fn key_with_one_existing_row_passes_validation() {
        let mut snap = snapshot_at(fixed_now());
        snap.event_row_counts
            .insert("event_9d02ddd940c43f87".to_string(), 1);
        sample_event()
            .validate("event_9d02ddd940c43f87", &snap)
            .unwrap();
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
    fn apply_replaces_existing_row_for_key() {
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        let lg = sources.join("img").join("lg");
        fs::create_dir_all(&lg).unwrap();
        fs::write(
            sources.join("img").join("img-sources.yaml"),
            "event_9d02ddd940c43f87:\n  0:\n    author: Old Author\n    license:\n      text: Old-License\n",
        )
        .unwrap();
        fs::write(lg.join("event_9d02ddd940c43f87_0.webp"), b"old-bytes").unwrap();
        // The multi-line quoted description before the matched row is the shape a naive
        // line-based replace would corrupt.
        fs::write(
            sources.join("events.csv"),
            "event_image,event_lat,event_lon,event_name,event_datetime_start_at,event_datetime_end_at,event_description,event_organising_org_id,event_image_author\n\
            /cdn/thumb/event_aaa_0.webp,48.1,11.5,Before,2026-06-15T16:00:00+02:00,2026-06-16T16:00:00+02:00,\"multi\nline, stays untouched\",1,Studi\n\
            /cdn/thumb/event_9d02ddd940c43f87_0.webp,48.0,11.0,Old Name,2026-06-01T10:00:00+02:00,2026-06-02T10:00:00+02:00,Old description.,99,Old Author\n\
            /cdn/thumb/event_bbb_0.webp,48.2,11.6,After,2026-06-15T16:00:00+02:00,2026-06-16T16:00:00+02:00,stays too,1,Studi\n",
        )
        .unwrap();

        let summary = sample_event()
            .apply("event_9d02ddd940c43f87", dir.path(), "branch")
            .unwrap();
        assert_snapshot!(summary, @r"
        update event `GARNIX Festival` (10.6.26 16:00 - 12.6.26 23:00, org `51897`)

        ![image showing event_9d02ddd940c43f87](https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/refs/heads/branch/data/sources/img/lg/event_9d02ddd940c43f87_0.webp)
        ");

        let csv = fs::read_to_string(sources.join("events.csv")).unwrap();
        assert_snapshot!(csv, @r#"
        event_image,event_lat,event_lon,event_name,event_datetime_start_at,event_datetime_end_at,event_description,event_organising_org_id,event_image_author
        /cdn/thumb/event_aaa_0.webp,48.1,11.5,Before,2026-06-15T16:00:00+02:00,2026-06-16T16:00:00+02:00,"multi
        line, stays untouched",1,Studi
        /cdn/thumb/event_9d02ddd940c43f87_0.webp,48.262908,11.669102,GARNIX Festival,2026-06-10T16:00:00+02:00,2026-06-12T23:00:00+02:00,"Live music, food trucks, and stands.",51897,Studi
        /cdn/thumb/event_bbb_0.webp,48.2,11.6,After,2026-06-15T16:00:00+02:00,2026-06-16T16:00:00+02:00,stays too,1,Studi
        "#);
    }

    #[test]
    fn update_replaces_image_files_and_metadata_idempotently() {
        let dir = tempfile::tempdir().unwrap();
        let sources = dir.path().join("data").join("sources");
        let lg = sources.join("img").join("lg");
        fs::create_dir_all(&lg).unwrap();
        // A legacy second slot plus a foreign key's entry: the update must drop the former
        // and leave the latter alone.
        fs::write(
            sources.join("img").join("img-sources.yaml"),
            "event_9d02ddd940c43f87:\n  0:\n    author: Old Author\n    license:\n      text: Old-License\n  1:\n    author: Stray Author\n    license:\n      text: Stray-License\nevent_bbb:\n  0:\n    author: Other\n    license:\n      text: CC0\n",
        )
        .unwrap();
        fs::write(lg.join("event_9d02ddd940c43f87_0.webp"), b"old-bytes").unwrap();
        fs::write(lg.join("event_9d02ddd940c43f87_1.webp"), b"stray-bytes").unwrap();
        fs::write(lg.join("event_bbb_0.webp"), b"other-bytes").unwrap();
        fs::write(
            sources.join("events.csv"),
            "event_image,event_lat,event_lon,event_name,event_datetime_start_at,event_datetime_end_at,event_description,event_organising_org_id,event_image_author\n\
            /cdn/thumb/event_9d02ddd940c43f87_0.webp,48.0,11.0,Old Name,2026-06-01T10:00:00+02:00,2026-06-02T10:00:00+02:00,Old description.,99,Old Author\n",
        )
        .unwrap();

        sample_event()
            .apply("event_9d02ddd940c43f87", dir.path(), "branch")
            .unwrap();

        let poster = fs::read(lg.join("event_9d02ddd940c43f87_0.webp")).unwrap();
        assert_ne!(poster, b"old-bytes");
        assert!(!lg.join("event_9d02ddd940c43f87_1.webp").exists());
        assert_eq!(
            fs::read(lg.join("event_bbb_0.webp")).unwrap(),
            b"other-bytes"
        );
        let yaml = fs::read_to_string(sources.join("img").join("img-sources.yaml")).unwrap();
        assert_snapshot!(yaml, @r"
        event_9d02ddd940c43f87:
          0:
            author: Studi
            license:
              text: CC-BY
        event_bbb:
          0:
            author: Other
            license:
              text: CC0
        ");

        // Resubmitting the identical request must not change the files again.
        let csv_after_first = fs::read(sources.join("events.csv")).unwrap();
        sample_event()
            .apply("event_9d02ddd940c43f87", dir.path(), "branch")
            .unwrap();
        assert_eq!(
            fs::read(lg.join("event_9d02ddd940c43f87_0.webp")).unwrap(),
            poster
        );
        assert_eq!(
            fs::read(sources.join("events.csv")).unwrap(),
            csv_after_first
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
