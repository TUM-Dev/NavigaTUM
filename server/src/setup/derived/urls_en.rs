use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default)]
struct RawUrl {
    key: String,
    url: Option<String>,
    text: Option<String>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("urls_en.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawUrl>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawUrl| match (col, field) {
        ("key", Field::Str(v)) => r.key.clone_from(v),
        ("url", Field::Str(v)) => r.url = Some(v.clone()),
        ("text", Field::Str(v)) => r.text = Some(v.clone()),
        _ => {}
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawUrl]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE urls_en")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE urls_en").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(tx: &mut Transaction<'_, Postgres>, r: &RawUrl) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO urls_en (key, url, text) VALUES ($1, $2, $3)",
        r.key,
        r.url,
        r.text,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
