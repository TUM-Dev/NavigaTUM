use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::DerivedTable;

#[derive(Debug, Default)]
pub struct RawParent {
    key: String,
    id: Option<String>,
    name: Option<String>,
}

pub struct Parents;

impl DerivedTable for Parents {
    const FILENAME: &'static str = "parents.parquet";
    const TABLE: &'static str = "parents";
    type Row = RawParent;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("key", Field::Str(v)) => r.key.clone_from(v),
            ("id", Field::Str(v)) => r.id = Some(v.clone()),
            ("name", Field::Str(v)) => r.name = Some(v.clone()),
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> sqlx::Result<()> {
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
