use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use sqlx::FromRow;

    use super::{RawImage, load_rows};
    use crate::setup::tests::PostgresTestContainer;

    #[derive(Debug, PartialEq, Eq, FromRow)]
    struct ImageRow {
        key: Option<String>,
        name: Option<String>,
        author_url: Option<String>,
        author_text: Option<String>,
        source_url: Option<String>,
        source_text: Option<String>,
        license_url: Option<String>,
        license_text: Option<String>,
    }

    const EXPECTED_FROM_DE: &str = "\
        WITH avail AS ( \
            SELECT key, jsonb_array_elements(data -> 'imgs') AS img \
            FROM de \
            WHERE jsonb_typeof(data -> 'imgs') = 'array' \
        ) \
        SELECT key, \
               img ->> 'name'              AS name, \
               img -> 'author'  ->> 'url'  AS author_url, \
               img -> 'author'  ->> 'text' AS author_text, \
               img -> 'source'  ->> 'url'  AS source_url, \
               img -> 'source'  ->> 'text' AS source_text, \
               img -> 'license' ->> 'url'  AS license_url, \
               img -> 'license' ->> 'text' AS license_text \
        FROM avail";

    const STABLE_ORDER: &str = " ORDER BY \
        key NULLS FIRST, \
        name NULLS FIRST, \
        author_url NULLS FIRST, \
        author_text NULLS FIRST, \
        source_url NULLS FIRST, \
        source_text NULLS FIRST, \
        license_url NULLS FIRST, \
        license_text NULLS FIRST";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn location_images_table_matches_jsonb_source() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let expected_query = format!("{EXPECTED_FROM_DE}{STABLE_ORDER}");
        let expected: Vec<ImageRow> = sqlx::query_as(&expected_query)
            .fetch_all(&pg.pool)
            .await
            .expect("expected rows from de JSONB");

        let raw_rows: Vec<RawImage> = expected
            .iter()
            .filter_map(|row| {
                row.key.as_ref().map(|key| RawImage {
                    key: key.clone(),
                    name: row.name.clone(),
                    author_url: row.author_url.clone(),
                    author_text: row.author_text.clone(),
                    source_url: row.source_url.clone(),
                    source_text: row.source_text.clone(),
                    license_url: row.license_url.clone(),
                    license_text: row.license_text.clone(),
                })
            })
            .collect();
        load_rows(&pg.pool, &raw_rows)
            .await
            .expect("load location_images from de-derived rows");

        let table_query = format!(
            "SELECT key, name, author_url, author_text, source_url, source_text, license_url, license_text \
             FROM location_images{STABLE_ORDER}"
        );
        let actual: Vec<ImageRow> = sqlx::query_as(&table_query)
            .fetch_all(&pg.pool)
            .await
            .expect("select from location_images");

        let expected_non_null: Vec<&ImageRow> =
            expected.iter().filter(|r| r.key.is_some()).collect();
        assert_eq!(expected_non_null.len(), actual.len(), "row count mismatch");
        for (e, a) in expected_non_null.iter().zip(actual.iter()) {
            assert_eq!(*e, a);
        }
    }
}
