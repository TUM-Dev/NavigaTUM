use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// A single column's value in an upsert.
pub(super) enum Field {
    /// Overwrite the column with this value.
    Set(String),
    /// Keep the existing row's value, or empty when inserting a new row. This
    /// lets an edit refresh only the columns it owns without clobbering a
    /// curated sibling column it was never given a value for.
    Keep,
}

/// Upsert a single row into a sorted-by-id CSV, keyed by its first column.
///
/// Full-overwrite variant: every column in `new_fields` is set. See
/// [`apply_csv_upsert_fields`] for the per-column semantics shared by both.
pub(super) fn apply_csv_upsert(
    key: &str,
    csv_file: &Path,
    new_fields: &[String],
) -> anyhow::Result<()> {
    let fields: Vec<Field> = new_fields.iter().map(|f| Field::Set(f.clone())).collect();
    apply_csv_upsert_fields(key, csv_file, &fields)
}

/// Upsert a single row into a sorted-by-id CSV, keyed by its first column.
///
/// The file is rewritten with `new_fields` inserted (or replacing the existing
/// row with the same key) so the `id` column stays sorted - matching how the
/// data pipeline keeps its source CSVs diff-friendly. `new_fields` only needs to
/// cover the leading columns the edit owns; any trailing columns the schema
/// defines are padded empty on insert and **preserved** from the existing row on
/// update. A [`Field::Keep`] column is likewise preserved from the existing row,
/// so an edit can refresh part of a record without dropping curated fields it
/// did not mean to touch.
pub(super) fn apply_csv_upsert_fields(
    key: &str,
    csv_file: &Path,
    new_fields: &[Field],
) -> anyhow::Result<()> {
    let temp_file = csv_file.with_extension("tmp");

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(csv_file)?;
    let header: Vec<String> = reader.headers()?.iter().map(ToString::to_string).collect();
    let col_count = header.len();

    {
        let output = File::create(&temp_file)?;
        let mut writer = BufWriter::new(output);
        writeln!(
            writer,
            "{}",
            header
                .iter()
                .map(|h| csv_escape(h))
                .collect::<Vec<_>>()
                .join(",")
        )?;

        let mut wrote_edit = false;

        for record in reader.records() {
            let record = record?;
            let existing_key = record.get(0).unwrap_or("");

            if !wrote_edit && existing_key >= key {
                let existing = (existing_key == key).then_some(&record);
                write_row(&mut writer, new_fields, col_count, existing)?;
                wrote_edit = true;
                if existing_key == key {
                    continue;
                }
            }
            if existing_key != key {
                writeln!(
                    writer,
                    "{}",
                    record.iter().map(csv_escape).collect::<Vec<_>>().join(",")
                )?;
            }
        }

        if !wrote_edit {
            write_row(&mut writer, new_fields, col_count, None)?;
        }
    }

    fs::rename(&temp_file, csv_file)?;
    Ok(())
}

fn write_row<W: Write>(
    writer: &mut W,
    new_fields: &[Field],
    col_count: usize,
    existing: Option<&csv::StringRecord>,
) -> io::Result<()> {
    let mut fields: Vec<String> = Vec::with_capacity(col_count);
    for i in 0..col_count {
        let value = match new_fields.get(i) {
            Some(Field::Set(v)) => csv_escape(v),
            // Columns the edit does not own (`Keep`, or beyond `new_fields`) keep
            // the existing row's value, or are padded empty when inserting.
            Some(Field::Keep) | None => existing
                .and_then(|r| r.get(i))
                .map(csv_escape)
                .unwrap_or_default(),
        };
        fields.push(value);
    }
    writeln!(writer, "{}", fields.join(","))
}

pub(super) fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
