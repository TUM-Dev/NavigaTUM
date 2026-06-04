use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
    super::decode_parquet_rows(body, |col, field, r: &mut RawUrl| {
        match (col, field) {
            ("key", Field::Str(v)) => r.key.clone_from(v),
            ("url", Field::Str(v)) => r.url = Some(v.clone()),
            ("text", Field::Str(v)) => r.text = Some(v.clone()),
            _ => {}
        }
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
    sqlx::query!("ANALYZE urls_en")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(
    tx: &mut Transaction<'_, Postgres>,
    r: &RawUrl,
) -> Result<(), sqlx::Error> {
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

#[cfg(test)]
mod tests {
    use sqlx::FromRow;

    use super::{RawUrl, load_rows};
    use crate::setup::tests::PostgresTestContainer;

    #[derive(Debug, PartialEq, Eq, FromRow)]
    struct UrlRow {
        key: Option<String>,
        url: Option<String>,
        text: Option<String>,
    }

    const EXPECTED_FROM_EN: &str = "\
        WITH unrolled AS ( \
            SELECT key, jsonb_array_elements(data -> 'props' -> 'links') AS link \
            FROM en \
            WHERE jsonb_typeof(data -> 'props' -> 'links') = 'array' \
        ) \
        SELECT key, \
               link -> 'url'  ->> 'en' AS url, \
               link -> 'text' ->> 'en' AS text \
        FROM unrolled";

    const STABLE_ORDER: &str = " ORDER BY \
        key NULLS FIRST, \
        url NULLS FIRST, \
        text NULLS FIRST";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn urls_en_table_matches_jsonb_source() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let expected_query = format!("{EXPECTED_FROM_EN}{STABLE_ORDER}");
        let expected: Vec<UrlRow> = sqlx::query_as(&expected_query)
            .fetch_all(&pg.pool)
            .await
            .expect("expected rows from en JSONB");

        let raw_rows: Vec<RawUrl> = expected
            .iter()
            .filter_map(|row| {
                row.key.as_ref().map(|key| RawUrl {
                    key: key.clone(),
                    url: row.url.clone(),
                    text: row.text.clone(),
                })
            })
            .collect();
        load_rows(&pg.pool, &raw_rows)
            .await
            .expect("load urls_en from en-derived rows");

        let table_query =
            format!("SELECT key, url, text FROM urls_en{STABLE_ORDER}");
        let actual: Vec<UrlRow> = sqlx::query_as(&table_query)
            .fetch_all(&pg.pool)
            .await
            .expect("select from urls_en");

        let expected_non_null: Vec<&UrlRow> =
            expected.iter().filter(|r| r.key.is_some()).collect();
        assert_eq!(expected_non_null.len(), actual.len(), "row count mismatch");
        for (e, a) in expected_non_null.iter().zip(actual.iter()) {
            assert_eq!(*e, a);
        }
    }
}
