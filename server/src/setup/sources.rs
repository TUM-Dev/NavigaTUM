use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawSource {
    key: String,
    url: Option<String>,
    name: Option<String>,
    patched: Option<bool>,
}

pub struct Sources;

impl Loader for Sources {
    const FILENAME: &'static str = "sources.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE sources";
    const ANALYZE_SQL: &'static str = "ANALYZE sources";
    type Row = RawSource;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("key", Field::Str(v)) => r.key.clone_from(v),
            ("url", Field::Str(v)) => r.url = Some(v.clone()),
            ("name", Field::Str(v)) => r.name = Some(v.clone()),
            ("patched", Field::Bool(v)) => r.patched = Some(*v),
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
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
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<Sources>(pool).await
}
