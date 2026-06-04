use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default)]
struct RawRankingFactors {
    id: String,
    rank_type: Option<i16>,
    rank_combined: Option<i16>,
    rank_usage: Option<i16>,
    rank_custom: Option<i16>,
    rank_boost: Option<i16>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("ranking_factors.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawRankingFactors>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawRankingFactors| {
        match (col, field) {
            ("id", Field::Str(v)) => r.id.clone_from(v),
            // SMALLINT in PG; the parquet physical type is INT32 even for
            // dataframely Int16. try_from drops out-of-range rows to None,
            // which the dataframely Int16 validation upstream should make
            // unreachable in practice.
            ("rank_type", Field::Int(v)) => r.rank_type = i16::try_from(*v).ok(),
            ("rank_combined", Field::Int(v)) => r.rank_combined = i16::try_from(*v).ok(),
            ("rank_usage", Field::Int(v)) => r.rank_usage = i16::try_from(*v).ok(),
            ("rank_custom", Field::Int(v)) => r.rank_custom = i16::try_from(*v).ok(),
            ("rank_boost", Field::Int(v)) => r.rank_boost = i16::try_from(*v).ok(),
            _ => {}
        }
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawRankingFactors]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE ranking_factors")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE ranking_factors")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(
    tx: &mut Transaction<'_, Postgres>,
    r: &RawRankingFactors,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO ranking_factors \
         (id, rank_type, rank_combined, rank_usage, rank_custom, rank_boost) \
         VALUES ($1, $2, $3, $4, $5, $6)",
        r.id,
        r.rank_type,
        r.rank_combined,
        r.rank_usage,
        r.rank_custom,
        r.rank_boost,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
