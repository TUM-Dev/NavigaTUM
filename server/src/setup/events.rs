use std::env;

use crate::setup::file_loader;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use parquet::file::reader::{FileReader as _, SerializedFileReader};
use parquet::record::Field;
use sqlx::postgres::PgQueryResult;
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Default, Debug)]
struct RawEvent {
    name: String,
    description: String,
    image: String,
    lat: f64,
    lon: f64,
    starts_at: String,
    ends_at: String,
    organising_org_id: i32,
}

struct DBEvent {
    name: String,
    description: String,
    image: String,
    lat: f64,
    lon: f64,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    organising_org_id: i32,
}

impl DBEvent {
    async fn store(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO events(name, description, image, coordinate, starts_at, ends_at, organising_org_id) \
             VALUES ($1, $2, $3, POINT($4, $5), $6, $7, $8)",
            self.name,
            self.description,
            self.image,
            self.lat,
            self.lon,
            self.starts_at,
            self.ends_at,
            self.organising_org_id,
        )
        .execute(&mut **tx)
        .await
    }
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("events.parquet", &cdn_url).await?;

    let reader = SerializedFileReader::new(Bytes::from(body))?;
    let mut events = Vec::new();
    for row in reader.get_row_iter(None)? {
        let row = row?;
        let mut raw = RawEvent::default();
        for (col_name, field) in row.get_column_iter() {
            match (col_name.as_str(), field) {
                ("name", Field::Str(v)) => raw.name.clone_from(v),
                ("description", Field::Str(v)) => raw.description.clone_from(v),
                ("image", Field::Str(v)) => raw.image.clone_from(v),
                ("lat", Field::Float(v)) => raw.lat = f64::from(*v),
                ("lat", Field::Double(v)) => raw.lat = *v,
                ("lon", Field::Float(v)) => raw.lon = f64::from(*v),
                ("lon", Field::Double(v)) => raw.lon = *v,
                ("starts_at", Field::Str(v)) => raw.starts_at.clone_from(v),
                ("ends_at", Field::Str(v)) => raw.ends_at.clone_from(v),
                ("organising_org_id", Field::Int(v)) => raw.organising_org_id = *v,
                _ => {}
            }
        }
        if raw.name.is_empty() {
            continue;
        }
        events.push(DBEvent {
            name: raw.name,
            description: raw.description,
            image: raw.image,
            lat: raw.lat,
            lon: raw.lon,
            starts_at: DateTime::parse_from_rfc3339(&raw.starts_at)?.with_timezone(&Utc),
            ends_at: DateTime::parse_from_rfc3339(&raw.ends_at)?.with_timezone(&Utc),
            organising_org_id: raw.organising_org_id,
        });
    }

    let mut tx = pool.begin().await?;
    clean(&mut tx).await?;
    for event in events {
        event.store(&mut tx).await?;
    }
    sqlx::query!("ANALYZE events").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn clean(tx: &mut Transaction<'_, Postgres>) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM events WHERE 1=1")
        .execute(&mut **tx)
        .await
}
