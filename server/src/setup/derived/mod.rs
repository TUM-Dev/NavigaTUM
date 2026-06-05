use std::env;
use std::future::Future;

use bytes::Bytes;
use parquet::file::reader::{FileReader as _, SerializedFileReader};
use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

pub(crate) mod location_images;
pub(crate) mod operators_de;
pub(crate) mod operators_en;
pub(crate) mod parents;
pub(crate) mod ranking_factors;
pub(crate) mod sources;
pub(crate) mod urls_de;
pub(crate) mod urls_en;
pub(crate) mod usages;

/// `insert` is per-table because `sqlx::query!` requires literal SQL.
/// TRUNCATE / ANALYZE are pre-baked as `&'static str` so [`run`] can issue
/// them via runtime [`sqlx::query`] without tripping sqlx 0.9's non-`'static`
/// SQL ban.
pub(super) trait DerivedTable {
    const FILENAME: &'static str;
    const TRUNCATE_SQL: &'static str;
    const ANALYZE_SQL: &'static str;
    type Row: Default + Send;

    fn parse_field(col: &str, field: &Field, row: &mut Self::Row);

    fn insert(
        tx: &mut Transaction<'_, Postgres>,
        row: &Self::Row,
    ) -> impl Future<Output = sqlx::Result<()>> + Send;
}

#[tracing::instrument(skip(pool), fields(filename = T::FILENAME))]
pub(super) async fn run<T: DerivedTable>(pool: PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download(T::FILENAME, &cdn_url).await?;
    let rows = decode_parquet_rows::<T::Row>(body, T::parse_field)?;
    let mut tx = pool.begin().await?;
    sqlx::query(T::TRUNCATE_SQL).execute(&mut *tx).await?;
    for r in &rows {
        T::insert(&mut tx, r).await?;
    }
    sqlx::query(T::ANALYZE_SQL).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

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
