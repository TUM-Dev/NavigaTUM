use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default)]
struct RawSource {
    key: String,
    url: Option<String>,
    name: Option<String>,
    patched: Option<bool>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("sources.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawSource>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawSource| match (col, field) {
        ("key", Field::Str(v)) => r.key.clone_from(v),
        ("url", Field::Str(v)) => r.url = Some(v.clone()),
        ("name", Field::Str(v)) => r.name = Some(v.clone()),
        ("patched", Field::Bool(v)) => r.patched = Some(*v),
        _ => {}
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawSource]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE sources")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE sources").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(tx: &mut Transaction<'_, Postgres>, r: &RawSource) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO sources (key, url, name, patched) VALUES ($1, $2, $3, $4)",
        r.key,
        r.url,
        r.name,
        r.patched,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

