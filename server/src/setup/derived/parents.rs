use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default)]
struct RawParent {
    key: String,
    id: Option<String>,
    name: Option<String>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("parents.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawParent>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawParent| match (col, field) {
        ("key", Field::Str(v)) => r.key.clone_from(v),
        ("id", Field::Str(v)) => r.id = Some(v.clone()),
        ("name", Field::Str(v)) => r.name = Some(v.clone()),
        _ => {}
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawParent]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE parents")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE parents").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(tx: &mut Transaction<'_, Postgres>, r: &RawParent) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO parents (key, id, name) VALUES ($1, $2, $3)",
        r.key,
        r.id,
        r.name,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
