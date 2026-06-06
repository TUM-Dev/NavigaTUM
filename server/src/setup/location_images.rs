use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawImage {
    key: String,
    name: Option<String>,
    author_url: Option<String>,
    author_text: Option<String>,
    source_url: Option<String>,
    source_text: Option<String>,
    license_url: Option<String>,
    license_text: Option<String>,
}

pub struct LocationImages;

impl Loader for LocationImages {
    const FILENAME: &'static str = "location_images.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE location_images";
    const ANALYZE_SQL: &'static str = "ANALYZE location_images";
    type Row = RawImage;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("key", Field::Str(v)) => r.key.clone_from(v),
            ("name", Field::Str(v)) => r.name = Some(v.clone()),
            ("author_url", Field::Str(v)) => r.author_url = Some(v.clone()),
            ("author_text", Field::Str(v)) => r.author_text = Some(v.clone()),
            ("source_url", Field::Str(v)) => r.source_url = Some(v.clone()),
            ("source_text", Field::Str(v)) => r.source_text = Some(v.clone()),
            ("license_url", Field::Str(v)) => r.license_url = Some(v.clone()),
            ("license_text", Field::Str(v)) => r.license_text = Some(v.clone()),
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
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
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<LocationImages>(pool).await
}
