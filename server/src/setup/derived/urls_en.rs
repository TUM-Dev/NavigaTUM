use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::DerivedTable;

#[derive(Debug, Default)]
pub struct RawUrl {
    key: String,
    url: Option<String>,
    text: Option<String>,
}

pub struct UrlsEn;

impl DerivedTable for UrlsEn {
    const FILENAME: &'static str = "urls_en.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE urls_en";
    const ANALYZE_SQL: &'static str = "ANALYZE urls_en";
    type Row = RawUrl;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("key", Field::Str(v)) => r.key.clone_from(v),
            ("url", Field::Str(v)) => r.url = Some(v.clone()),
            ("text", Field::Str(v)) => r.text = Some(v.clone()),
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> sqlx::Result<()> {
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
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<UrlsEn>(pool).await
}
