use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// Upsert a single row into a sorted-by-id CSV, keyed by its first column.
///
/// The file is rewritten with `new_fields` inserted (or replacing the existing
/// row with the same key) so the `id` column stays sorted - matching how the
/// data pipeline keeps its source CSVs diff-friendly. `new_fields` only needs to
/// cover the leading columns the edit owns; any trailing columns the schema
/// defines are padded empty on insert and **preserved** from the existing row on
/// update, so an edit can refresh part of a record without dropping curated
/// optional fields.
pub(super) fn apply_csv_upsert(
    key: &str,
    csv_file: &Path,
    new_fields: &[String],
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
                let extras: Option<Vec<String>> = (existing_key == key).then(|| {
                    record
                        .iter()
                        .skip(new_fields.len())
                        .map(ToString::to_string)
                        .collect()
                });
                write_padded_row(&mut writer, new_fields, col_count, extras.as_deref())?;
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
            write_padded_row(&mut writer, new_fields, col_count, None)?;
        }
    }

    fs::rename(&temp_file, csv_file)?;
    Ok(())
}

fn write_padded_row<W: Write>(
    writer: &mut W,
    new_fields: &[String],
    col_count: usize,
    extras: Option<&[String]>,
) -> io::Result<()> {
    let mut fields: Vec<String> = new_fields.iter().map(|f| csv_escape(f)).collect();
    let known = new_fields.len();
    if col_count > known {
        let trailing = col_count - known;
        if let Some(ex) = extras {
            for i in 0..trailing {
                fields.push(ex.get(i).map(|s| csv_escape(s)).unwrap_or_default());
            }
        } else {
            for _ in 0..trailing {
                fields.push(String::new());
            }
        }
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
