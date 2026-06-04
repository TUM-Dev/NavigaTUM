use bytes::Bytes;
use parquet::file::reader::{FileReader as _, SerializedFileReader};
use parquet::record::Field;

pub(crate) mod location_images;
pub(crate) mod operators_de;
pub(crate) mod operators_en;
pub(crate) mod parents;
pub(crate) mod ranking_factors;
pub(crate) mod sources;
pub(crate) mod urls_de;
pub(crate) mod urls_en;
pub(crate) mod usages;

fn decode_parquet_rows<T: Default>(
    body: Vec<u8>,
    extract: impl Fn(&str, &Field, &mut T),
) -> anyhow::Result<Vec<T>> {
    let reader = SerializedFileReader::new(Bytes::from(body))?;
    let mut rows = Vec::new();
    for row in reader.get_row_iter(None)? {
        let row = row?;
        let mut r = T::default();
        for (col_name, field) in row.get_column_iter() {
            extract(col_name.as_str(), field, &mut r);
        }
        rows.push(r);
    }
    Ok(rows)
}
