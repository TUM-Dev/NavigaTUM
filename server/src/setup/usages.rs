use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawUsage {
    name: String,
    din_277: Option<String>,
    din_277_desc: Option<String>,
}

pub struct Usages;

impl Loader for Usages {
    const FILENAME: &'static str = "usages.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE usages";
    const ANALYZE_SQL: &'static str = "ANALYZE usages";
    type Row = RawUsage;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("name", Field::Str(v)) => r.name.clone_from(v),
            ("din_277", Field::Str(v)) => r.din_277 = Some(v.clone()),
            ("din_277_desc", Field::Str(v)) => r.din_277_desc = Some(v.clone()),
            _ => {}
        }
    }

    /// `usage_id = hashtext(name)` is computed in SQL - Postgres' hashtext
    /// is not reproducible from Polars.
    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
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
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<Usages>(pool).await
}
