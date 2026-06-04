use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
    super::decode_parquet_rows(body, |col, field, r: &mut RawOperator| {
        match (col, field) {
            ("id", Field::Int(v)) => r.id = *v,
            ("url", Field::Str(v)) => r.url = Some(v.clone()),
            ("code", Field::Str(v)) => r.code = Some(v.clone()),
            ("name", Field::Str(v)) => r.name = Some(v.clone()),
            _ => {}
        }
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

#[cfg(test)]
mod tests {
    use sqlx::FromRow;

    use super::{RawOperator, load_rows};
    use crate::setup::tests::PostgresTestContainer;

    #[derive(Debug, PartialEq, Eq, FromRow)]
    struct OperatorRow {
        id: Option<i32>,
        url: Option<String>,
        code: Option<String>,
        name: Option<String>,
    }

    // NULL ids would collapse to a single all-NULL row under the legacy
    // view's DISTINCT, but the table's PK can't accept them.
    const EXPECTED_FROM_DE: &str = "\
        SELECT DISTINCT (data -> 'props' -> 'operator' ->> 'id')::integer AS id, \
                        data -> 'props' -> 'operator' ->> 'url'           AS url, \
                        data -> 'props' -> 'operator' ->> 'code'          AS code, \
                        data -> 'props' -> 'operator' ->> 'name'          AS name \
        FROM de \
        WHERE (data -> 'props' -> 'operator' ->> 'id') IS NOT NULL";

    const STABLE_ORDER: &str = " ORDER BY id";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn operators_de_table_matches_jsonb_source() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let expected_query = format!("{EXPECTED_FROM_DE}{STABLE_ORDER}");
        let expected: Vec<OperatorRow> = sqlx::query_as(&expected_query)
            .fetch_all(&pg.pool)
            .await
            .expect("expected rows from de JSONB");

        let raw_rows: Vec<RawOperator> = expected
            .iter()
            .filter_map(|row| {
                row.id.map(|id| RawOperator {
                    id,
                    url: row.url.clone(),
                    code: row.code.clone(),
                    name: row.name.clone(),
                })
            })
            .collect();
        load_rows(&pg.pool, &raw_rows)
            .await
            .expect("load operators_de from de-derived rows");

        let table_query =
            format!("SELECT id, url, code, name FROM operators_de{STABLE_ORDER}");
        let actual: Vec<OperatorRow> = sqlx::query_as(&table_query)
            .fetch_all(&pg.pool)
            .await
            .expect("select from operators_de");

        assert_eq!(expected.len(), actual.len(), "row count mismatch");
        for (e, a) in expected.iter().zip(actual.iter()) {
            assert_eq!(e, a);
        }
    }
}
