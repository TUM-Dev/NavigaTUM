use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use sqlx::FromRow;

    use super::{RawParent, load_rows};
    use crate::setup::tests::PostgresTestContainer;

    #[derive(Debug, PartialEq, Eq, FromRow)]
    struct ParentRow {
        key: Option<String>,
        id: Option<String>,
        name: Option<String>,
    }

    // parent_names entries are mixed: plain strings for normal parents,
    // `{de, en}` objects for the synthetic root. COALESCE handles both.
    const EXPECTED_FROM_DE: &str = "\
        WITH paired AS ( \
            SELECT key, \
                   jsonb_array_elements_text(data -> 'parents')      AS id, \
                   jsonb_array_elements(data -> 'parent_names')      AS name_jsonb \
            FROM de \
            WHERE jsonb_typeof(data -> 'parents') = 'array' \
              AND jsonb_typeof(data -> 'parent_names') = 'array' \
        ) \
        SELECT key, id, \
               COALESCE(name_jsonb ->> 'de', name_jsonb #>> '{}') AS name \
        FROM paired";

    const STABLE_ORDER: &str = " ORDER BY \
        key NULLS FIRST, \
        id NULLS FIRST, \
        name NULLS FIRST";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn parents_table_matches_jsonb_source() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let expected_query = format!("{EXPECTED_FROM_DE}{STABLE_ORDER}");
        let expected: Vec<ParentRow> = sqlx::query_as(&expected_query)
            .fetch_all(&pg.pool)
            .await
            .expect("expected rows from de JSONB");

        let raw_rows: Vec<RawParent> = expected
            .iter()
            .filter_map(|row| {
                row.key.as_ref().map(|key| RawParent {
                    key: key.clone(),
                    id: row.id.clone(),
                    name: row.name.clone(),
                })
            })
            .collect();
        load_rows(&pg.pool, &raw_rows)
            .await
            .expect("load parents from de-derived rows");

        let table_query = format!("SELECT key, id, name FROM parents{STABLE_ORDER}");
        let actual: Vec<ParentRow> = sqlx::query_as(&table_query)
            .fetch_all(&pg.pool)
            .await
            .expect("select from parents");

        let expected_non_null: Vec<&ParentRow> =
            expected.iter().filter(|r| r.key.is_some()).collect();
        assert_eq!(expected_non_null.len(), actual.len(), "row count mismatch");
        for (e, a) in expected_non_null.iter().zip(actual.iter()) {
            assert_eq!(*e, a);
        }
    }
}
