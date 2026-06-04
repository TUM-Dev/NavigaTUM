use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default)]
struct RawOperator {
    id: i32,
    url: Option<String>,
    code: Option<String>,
    name: Option<String>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("operators_de.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawOperator>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawOperator| match (col, field) {
        ("id", Field::Int(v)) => r.id = *v,
        ("url", Field::Str(v)) => r.url = Some(v.clone()),
        ("code", Field::Str(v)) => r.code = Some(v.clone()),
        ("name", Field::Str(v)) => r.name = Some(v.clone()),
        _ => {}
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawOperator]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE operators_de")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE operators_de")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(
    tx: &mut Transaction<'_, Postgres>,
    r: &RawOperator,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO operators_de (id, url, code, name) VALUES ($1, $2, $3, $4)",
        r.id,
        r.url,
        r.code,
        r.name,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

