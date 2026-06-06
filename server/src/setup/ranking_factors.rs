use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawRankingFactors {
    id: String,
    rank_type: Option<i16>,
    rank_combined: Option<i16>,
    rank_usage: Option<i16>,
    rank_custom: Option<i16>,
    rank_boost: Option<i16>,
}

pub struct RankingFactors;

impl Loader for RankingFactors {
    const FILENAME: &'static str = "ranking_factors.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE ranking_factors";
    const ANALYZE_SQL: &'static str = "ANALYZE ranking_factors";
    type Row = RawRankingFactors;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
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
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
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
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<RankingFactors>(pool).await
}
