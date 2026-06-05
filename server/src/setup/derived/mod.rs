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

/// `insert` is per-table because `sqlx::query!` requires literal SQL (so the
/// column list and parameter types cannot be expressed generically). TRUNCATE
/// and ANALYZE are param-less DDL on a compile-time-known [`Self::TABLE`] and
/// are issued via runtime [`sqlx::query`] from the shared [`run`].
pub(super) trait DerivedTable {
    const FILENAME: &'static str;
    /// Interpolated directly into runtime SQL; must be a `&'static` identifier
    /// to keep TRUNCATE / ANALYZE injection-safe.
    const TABLE: &'static str;
    type Row: Default + Send;

    fn parse_field(col: &str, field: &Field, row: &mut Self::Row);

    fn insert(
        tx: &mut Transaction<'_, Postgres>,
        row: &Self::Row,
    ) -> impl Future<Output = sqlx::Result<()>> + Send;
}

#[tracing::instrument(skip(pool), fields(table = T::TABLE))]
pub(super) async fn run<T: DerivedTable>(pool: PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download(T::FILENAME, &cdn_url).await?;
    let rows = decode_parquet_rows::<T::Row>(body, T::parse_field)?;
    let mut tx = pool.begin().await?;
    sqlx::query(&format!("TRUNCATE TABLE {}", T::TABLE))
        .execute(&mut *tx)
        .await?;
    for r in &rows {
        T::insert(&mut tx, r).await?;
    }
    sqlx::query(&format!("ANALYZE {}", T::TABLE))
        .execute(&mut *tx)
        .await?;
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
