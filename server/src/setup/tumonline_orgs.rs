use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawOrg {
    org_id: i32,
    code: String,
    name_de: String,
    name_en: String,
    path_de: Option<String>,
    path_en: Option<String>,
}

pub struct TumonlineOrgs;

impl Loader for TumonlineOrgs {
    const FILENAME: &'static str = "tumonline_orgs.parquet";
    // CASCADE because events.organising_org_id REFERENCES tumonline_orgs.org_id.
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE tumonline_orgs RESTART IDENTITY CASCADE";
    const ANALYZE_SQL: &'static str = "ANALYZE tumonline_orgs";
    type Row = RawOrg;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("org_id", Field::Int(v)) => r.org_id = *v,
            ("code", Field::Str(v)) => r.code.clone_from(v),
            ("name_de", Field::Str(v)) => r.name_de.clone_from(v),
            ("name_en", Field::Str(v)) => r.name_en.clone_from(v),
            ("path_de", Field::Str(v)) => r.path_de = Some(v.clone()),
            ("path_en", Field::Str(v)) => r.path_en = Some(v.clone()),
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO tumonline_orgs(org_id, code, name_de, name_en, path_de, path_en) \
             VALUES ($1, $2, $3, $4, $5, $6)",
            r.org_id,
            r.code,
            r.name_de,
            r.name_en,
            r.path_de,
            r.path_en,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<TumonlineOrgs>(pool).await
}
