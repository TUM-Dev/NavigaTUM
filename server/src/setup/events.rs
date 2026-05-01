use crate::setup::file_loader;
use chrono::{DateTime, Utc};
use polars::prelude::*;
use std::io::Write;
use tempfile::tempfile;

struct DBEvent {
    name: String,
    description: Option<String>,
    image: Option<String>,
    lat: f64,
    lon: f64,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    organising_org_id: i32,
}

impl DBEvent {
    async fn store(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
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
pub async fn setup(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("events.parquet", &cdn_url).await?;

    let mut file = tempfile()?;
    file.write_all(&body)?;

    let df = ParquetReader::new(&mut file).finish()?;

    let name_col = df.column("name")?.str()?;
    let description_col = df.column("description")?.str()?;
    let image_col = df.column("image")?.str()?;
    let lat_col = df.column("lat")?.f64()?;
    let lon_col = df.column("lon")?.f64()?;
    let starts_at_col = df.column("starts_at")?.str()?;
    let ends_at_col = df.column("ends_at")?.str()?;
    let organising_org_id_col = df.column("organising_org_id")?.i32()?;

    let mut events = Vec::with_capacity(df.height());
    for i in 0..df.height() {
        let Some(name) = name_col.get(i) else { continue };
        let Some(starts_raw) = starts_at_col.get(i) else { continue };
        let Some(ends_raw) = ends_at_col.get(i) else { continue };
        let Some(lat) = lat_col.get(i) else { continue };
        let Some(lon) = lon_col.get(i) else { continue };
        let Some(organising_org_id) = organising_org_id_col.get(i) else { continue };

        let starts_at = DateTime::parse_from_rfc3339(starts_raw)?.with_timezone(&Utc);
        let ends_at = DateTime::parse_from_rfc3339(ends_raw)?.with_timezone(&Utc);

        events.push(DBEvent {
            name: name.to_string(),
            description: description_col.get(i).map(str::to_string),
            image: image_col.get(i).map(str::to_string),
            lat,
            lon,
            starts_at,
            ends_at,
            organising_org_id,
        });
    }

    let mut tx = pool.begin().await?;
    clean(&mut tx).await?;
    for event in events {
        event.store(&mut tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn clean(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM events WHERE 1=1")
        .execute(&mut **tx)
        .await
}
