use chrono::{DateTime, Utc};
use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawEvent {
    name: String,
    description: String,
    image: String,
    image_author: String,
    lat: f64,
    lon: f64,
    starts_at: String,
    ends_at: String,
    appears_at: String,
    organising_org_id: i32,
}

pub struct Events;

impl Loader for Events {
    const FILENAME: &'static str = "events.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE events RESTART IDENTITY";
    const ANALYZE_SQL: &'static str = "ANALYZE events";
    type Row = RawEvent;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            ("name", Field::Str(v)) => r.name.clone_from(v),
            ("description", Field::Str(v)) => r.description.clone_from(v),
            ("image", Field::Str(v)) => r.image.clone_from(v),
            ("image_author", Field::Str(v)) => r.image_author.clone_from(v),
            ("lat", Field::Float(v)) => r.lat = f64::from(*v),
            ("lat", Field::Double(v)) => r.lat = *v,
            ("lon", Field::Float(v)) => r.lon = f64::from(*v),
            ("lon", Field::Double(v)) => r.lon = *v,
            ("starts_at", Field::Str(v)) => r.starts_at.clone_from(v),
            ("ends_at", Field::Str(v)) => r.ends_at.clone_from(v),
            ("appears_at", Field::Str(v)) => r.appears_at.clone_from(v),
            ("organising_org_id", Field::Int(v)) => r.organising_org_id = *v,
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
        if r.name.is_empty() {
            return Ok(());
        }
        let starts_at = DateTime::parse_from_rfc3339(&r.starts_at)?.with_timezone(&Utc);
        let ends_at = DateTime::parse_from_rfc3339(&r.ends_at)?.with_timezone(&Utc);
        let appears_at = DateTime::parse_from_rfc3339(&r.appears_at)?.with_timezone(&Utc);
        sqlx::query!(
            "INSERT INTO events(name, description, image, image_author, coordinate, starts_at, ends_at, appears_at, organising_org_id) \
             VALUES ($1, $2, $3, $4, POINT($5, $6), $7, $8, $9, $10)",
            r.name,
            r.description,
            r.image,
            r.image_author,
            r.lat,
            r.lon,
            starts_at,
            ends_at,
            appears_at,
            r.organising_org_id,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<Events>(pool).await
}
