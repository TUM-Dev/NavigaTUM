use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default)]
struct RawUsage {
    name: String,
    din_277: Option<String>,
    din_277_desc: Option<String>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("usages.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawUsage>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawUsage| match (col, field) {
        ("name", Field::Str(v)) => r.name.clone_from(v),
        ("din_277", Field::Str(v)) => r.din_277 = Some(v.clone()),
        ("din_277_desc", Field::Str(v)) => r.din_277_desc = Some(v.clone()),
        _ => {}
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawUsage]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE usages")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE usages").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(tx: &mut Transaction<'_, Postgres>, r: &RawUsage) -> Result<(), sqlx::Error> {
    // `usage_id = hashtext(name)` is computed in SQL - Postgres' hashtext
    // is not reproducible from Polars.
    sqlx::query!(
        "INSERT INTO usages (usage_id, name, din_277, din_277_desc) \
         VALUES (hashtext($1), $1, $2, $3)",
        r.name,
        r.din_277,
        r.din_277_desc,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
