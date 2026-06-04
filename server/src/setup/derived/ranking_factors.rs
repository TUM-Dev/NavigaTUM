use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RawRankingFactors {
    id: String,
    rank_type: Option<i32>,
    rank_combined: Option<i32>,
    rank_usage: Option<i32>,
    rank_custom: Option<i32>,
    rank_boost: Option<i32>,
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
            ("rank_type", Field::Int(v)) => r.rank_type = Some(*v),
            ("rank_combined", Field::Int(v)) => r.rank_combined = Some(*v),
            ("rank_usage", Field::Int(v)) => r.rank_usage = Some(*v),
            ("rank_custom", Field::Int(v)) => r.rank_custom = Some(*v),
            ("rank_boost", Field::Int(v)) => r.rank_boost = Some(*v),
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

#[cfg(test)]
mod tests {
    use sqlx::FromRow;

    use super::{RawRankingFactors, load_rows};
    use crate::setup::tests::PostgresTestContainer;

    #[derive(Debug, PartialEq, Eq, FromRow)]
    struct RankingFactorsRow {
        id: Option<String>,
        rank_type: Option<i32>,
        rank_combined: Option<i32>,
        rank_usage: Option<i32>,
        rank_custom: Option<i32>,
        rank_boost: Option<i32>,
    }

    const EXPECTED_FROM_DE: &str = "\
        SELECT DISTINCT data ->> 'id'                                            AS id, \
                        (data -> 'ranking_factors' ->> 'rank_type')::integer     AS rank_type, \
                        (data -> 'ranking_factors' ->> 'rank_combined')::integer AS rank_combined, \
                        (data -> 'ranking_factors' ->> 'rank_usage')::integer    AS rank_usage, \
                        (data -> 'ranking_factors' ->> 'rank_custom')::integer   AS rank_custom, \
                        (data -> 'ranking_factors' ->> 'rank_boost')::integer    AS rank_boost \
        FROM de";

    const STABLE_ORDER: &str = " ORDER BY \
        id NULLS FIRST, \
        rank_type NULLS FIRST, \
        rank_combined NULLS FIRST, \
        rank_usage NULLS FIRST, \
        rank_custom NULLS FIRST, \
        rank_boost NULLS FIRST";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn ranking_factors_table_matches_jsonb_source() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let expected_query = format!("{EXPECTED_FROM_DE}{STABLE_ORDER}");
        let expected: Vec<RankingFactorsRow> = sqlx::query_as(&expected_query)
            .fetch_all(&pg.pool)
            .await
            .expect("expected rows from de JSONB");

        let raw_rows: Vec<RawRankingFactors> = expected
            .iter()
            .filter_map(|row| {
                row.id.as_ref().map(|id| RawRankingFactors {
                    id: id.clone(),
                    rank_type: row.rank_type,
                    rank_combined: row.rank_combined,
                    rank_usage: row.rank_usage,
                    rank_custom: row.rank_custom,
                    rank_boost: row.rank_boost,
                })
            })
            .collect();
        load_rows(&pg.pool, &raw_rows)
            .await
            .expect("load ranking_factors from de-derived rows");

        let table_query = format!(
            "SELECT id, rank_type, rank_combined, rank_usage, rank_custom, rank_boost \
             FROM ranking_factors{STABLE_ORDER}"
        );
        let actual: Vec<RankingFactorsRow> = sqlx::query_as(&table_query)
            .fetch_all(&pg.pool)
            .await
            .expect("select from ranking_factors table");

        let expected_non_null: Vec<&RankingFactorsRow> =
            expected.iter().filter(|r| r.id.is_some()).collect();
        assert_eq!(expected_non_null.len(), actual.len(), "row count mismatch");
        for (e, a) in expected_non_null.iter().zip(actual.iter()) {
            assert_eq!(*e, a);
        }
    }
}
