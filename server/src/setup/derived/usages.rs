use std::env;

use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use crate::setup::file_loader;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct RawUsage {
    name: String,
    din_277: Option<String>,
    din_277_desc: Option<String>,
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("usages.parquet", &cdn_url).await?;
    let rows = parse_parquet(body)?;
    load_rows(pool, &rows).await
}

fn parse_parquet(body: Vec<u8>) -> anyhow::Result<Vec<RawUsage>> {
    super::decode_parquet_rows(body, |col, field, r: &mut RawUsage| {
        match (col, field) {
            ("name", Field::Str(v)) => r.name.clone_from(v),
            ("din_277", Field::Str(v)) => r.din_277 = Some(v.clone()),
            ("din_277_desc", Field::Str(v)) => r.din_277_desc = Some(v.clone()),
            _ => {}
        }
    })
}

async fn load_rows(pool: &PgPool, rows: &[RawUsage]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("TRUNCATE TABLE usages")
        .execute(&mut *tx)
        .await?;
    for r in rows {
        insert_row(&mut tx, r).await?;
    }
    sqlx::query!("ANALYZE usages")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_row(
    tx: &mut Transaction<'_, Postgres>,
    r: &RawUsage,
) -> Result<(), sqlx::Error> {
    // `usage_id = hashtext(name)` is computed in SQL — Postgres' hashtext
    // is not reproducible from Polars.
    sqlx::query!(
        "INSERT INTO usages (usage_id, name, din_277, din_277_desc) \
         VALUES (hashtext($1), $1, $2, $3)",
        r.name,
        r.din_277,
        r.din_277_desc,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::FromRow;

    use super::{RawUsage, load_rows};
    use crate::setup::tests::PostgresTestContainer;

    #[derive(Debug, PartialEq, Eq, FromRow)]
    struct UsageRow {
        usage_id: Option<i32>,
        name: Option<String>,
        din_277: Option<String>,
        din_277_desc: Option<String>,
    }

    const EXPECTED_FROM_DE_EN: &str = "\
        SELECT hashtext(data -> 'usage' ->> 'name') AS usage_id, \
               data -> 'usage' ->> 'name'           AS name, \
               data -> 'usage' ->> 'din_277'        AS din_277, \
               data -> 'usage' ->> 'din_277_desc'   AS din_277_desc \
        FROM de WHERE data -> 'usage' ->> 'name' IS NOT NULL \
        UNION \
        SELECT hashtext(data -> 'usage' ->> 'name') AS usage_id, \
               data -> 'usage' ->> 'name'           AS name, \
               data -> 'usage' ->> 'din_277'        AS din_277, \
               data -> 'usage' ->> 'din_277_desc'   AS din_277_desc \
        FROM en WHERE data -> 'usage' ->> 'name' IS NOT NULL";

    const STABLE_ORDER: &str = " ORDER BY \
        usage_id NULLS FIRST, \
        name NULLS FIRST, \
        din_277 NULLS FIRST, \
        din_277_desc NULLS FIRST";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn usages_table_matches_jsonb_source() {
        let pg = PostgresTestContainer::new().await;
        pg.load_data_retrying().await;

        let expected_query = format!("{EXPECTED_FROM_DE_EN}{STABLE_ORDER}");
        let expected: Vec<UsageRow> = sqlx::query_as(&expected_query)
            .fetch_all(&pg.pool)
            .await
            .expect("expected rows from de/en JSONB");

        let raw_rows: Vec<RawUsage> = expected
            .iter()
            .filter_map(|row| {
                row.name.as_ref().map(|name| RawUsage {
                    name: name.clone(),
                    din_277: row.din_277.clone(),
                    din_277_desc: row.din_277_desc.clone(),
                })
            })
            .collect();
        load_rows(&pg.pool, &raw_rows)
            .await
            .expect("load usages from de/en-derived rows");

        let table_query = format!(
            "SELECT usage_id, name, din_277, din_277_desc FROM usages{STABLE_ORDER}"
        );
        let actual: Vec<UsageRow> = sqlx::query_as(&table_query)
            .fetch_all(&pg.pool)
            .await
            .expect("select from usages");

        let expected_non_null: Vec<&UsageRow> =
            expected.iter().filter(|r| r.name.is_some()).collect();
        assert_eq!(expected_non_null.len(), actual.len(), "row count mismatch");
        for (e, a) in expected_non_null.iter().zip(actual.iter()) {
            assert_eq!(*e, a);
        }
    }
}
