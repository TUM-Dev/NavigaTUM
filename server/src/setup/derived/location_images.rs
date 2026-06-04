use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default)]
struct RawImage {
    key: String,
    name: Option<String>,
    author_url: Option<String>,
    author_text: Option<String>,
    source_url: Option<String>,
    source_text: Option<String>,
    license_url: Option<String>,
    license_text: Option<String>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("location_images.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawImage>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawImage| match (col, field) {
        ("key", Field::Str(v)) => r.key.clone_from(v),
        ("name", Field::Str(v)) => r.name = Some(v.clone()),
        ("author_url", Field::Str(v)) => r.author_url = Some(v.clone()),
        ("author_text", Field::Str(v)) => r.author_text = Some(v.clone()),
        ("source_url", Field::Str(v)) => r.source_url = Some(v.clone()),
        ("source_text", Field::Str(v)) => r.source_text = Some(v.clone()),
        ("license_url", Field::Str(v)) => r.license_url = Some(v.clone()),
        ("license_text", Field::Str(v)) => r.license_text = Some(v.clone()),
        _ => {}
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawImage]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE location_images")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE location_images")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(tx: &mut Transaction<'_, Postgres>, r: &RawImage) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO location_images (\
            key, name, \
            author_url, author_text, \
            source_url, source_text, \
            license_url, license_text\
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        r.key,
        r.name,
        r.author_url,
        r.author_text,
        r.source_url,
        r.source_text,
        r.license_url,
        r.license_text,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
