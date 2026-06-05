use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawParent {
    key: String,
    id: Option<String>,
    name: Option<String>,
}

pub struct Parents;

impl Loader for Parents {
    const FILENAME: &'static str = "parents.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE parents";
    const ANALYZE_SQL: &'static str = "ANALYZE parents";
    type Row = RawParent;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("key", Field::Str(v)) => r.key.clone_from(v),
            ("id", Field::Str(v)) => r.id = Some(v.clone()),
            ("name", Field::Str(v)) => r.name = Some(v.clone()),
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
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
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<Parents>(pool).await
}
