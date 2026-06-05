use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::DerivedTable;

#[derive(Debug, Default)]
pub struct RawOperator {
    id: i32,
    url: Option<String>,
    code: Option<String>,
    name: Option<String>,
}

pub struct OperatorsDe;

impl DerivedTable for OperatorsDe {
    const FILENAME: &'static str = "operators_de.parquet";
    const TABLE: &'static str = "operators_de";
    type Row = RawOperator;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("id", Field::Int(v)) => r.id = *v,
            ("url", Field::Str(v)) => r.url = Some(v.clone()),
            ("code", Field::Str(v)) => r.code = Some(v.clone()),
            ("name", Field::Str(v)) => r.name = Some(v.clone()),
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> sqlx::Result<()> {
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
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<OperatorsDe>(pool).await
}
