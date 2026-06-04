use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
    super::decode_parquet_rows(body, |col, field, r: &mut RawSource| {
        match (col, field) {
            ("key", Field::Str(v)) => r.key.clone_from(v),
            ("url", Field::Str(v)) => r.url = Some(v.clone()),
            ("name", Field::Str(v)) => r.name = Some(v.clone()),
            ("patched", Field::Bool(v)) => r.patched = Some(*v),
            _ => {}
        }
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
    sqlx::query!("ANALYZE sources")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(
    tx: &mut Transaction<'_, Postgres>,
    r: &RawSource,
) -> Result<(), sqlx::Error> {
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

#[cfg(test)]
mod tests {
    use sqlx::FromRow;

    use super::{RawSource, load_rows};
    use crate::setup::tests::PostgresTestContainer;

    #[derive(Debug, PartialEq, Eq, FromRow)]
    struct SourceRow {
        key: Option<String>,
        url: Option<String>,
        name: Option<String>,
        patched: Option<bool>,
    }

    const EXPECTED_FROM_DE: &str = "\
        WITH unrolled_sources(key, source, patched) AS ( \
            SELECT key, \
                   jsonb_array_elements(data -> 'sources' -> 'base') AS source, \
                   (data -> 'sources' ->> 'patched')::bool AS patched \
            FROM de \
            WHERE jsonb_typeof(data -> 'sources' -> 'base') = 'array' \
        ) \
        SELECT key, \
               source ->> 'url'  AS url, \
               source ->> 'name' AS name, \
               patched \
        FROM unrolled_sources";

    const STABLE_ORDER: &str = " ORDER BY \
        key NULLS FIRST, \
        name NULLS FIRST, \
        url NULLS FIRST, \
        patched NULLS FIRST";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn sources_table_matches_jsonb_source() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let expected_query = format!("{EXPECTED_FROM_DE}{STABLE_ORDER}");
        let expected: Vec<SourceRow> = sqlx::query_as(&expected_query)
            .fetch_all(&pg.pool)
            .await
            .expect("expected rows from de JSONB");

        let raw_rows: Vec<RawSource> = expected
            .iter()
            .filter_map(|row| {
                row.key.as_ref().map(|key| RawSource {
                    key: key.clone(),
                    url: row.url.clone(),
                    name: row.name.clone(),
                    patched: row.patched,
                })
            })
            .collect();
        load_rows(&pg.pool, &raw_rows)
            .await
            .expect("load sources from de-derived rows");

        let table_query =
            format!("SELECT key, url, name, patched FROM sources{STABLE_ORDER}");
        let actual: Vec<SourceRow> = sqlx::query_as(&table_query)
            .fetch_all(&pg.pool)
            .await
            .expect("select from sources");

        let expected_non_null: Vec<&SourceRow> =
            expected.iter().filter(|r| r.key.is_some()).collect();
        assert_eq!(expected_non_null.len(), actual.len(), "row count mismatch");
        for (e, a) in expected_non_null.iter().zip(actual.iter()) {
            assert_eq!(*e, a);
        }
    }
}
