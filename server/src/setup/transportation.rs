use polars::prelude::*;
use serde::Deserialize;
use std::io::Write;
use tempfile::tempfile;

#[derive(Deserialize, Default, Debug)]
struct Station {
    dhid: String,
    parent: Option<String>,
    name: String,
    lat: f64,
    lon: f64,
}

struct DBStation {
    parent: Option<String>,
    id: String,
    name: String,
    lat: f64,
    lon: f64,
}

impl DBStation {
    fn from_station(station: Station) -> Self {
        Self {
            parent: station.parent,
            id: station.dhid,
            name: station.name,
            lat: station.lat,
            lon: station.lon,
        }
    }
    async fn store(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO transportation_stations(parent,id,name,coordinate)\
            VALUES ($1,$2,$3,POINT($4,$5))",
            self.parent,
            self.id,
            self.name,
            self.lat,
            self.lon
        )
        .execute(&mut **tx)
        .await
    }
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    // Download the parquet file
    let url = "https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/main/data/external/results/public_transport.parquet";
    let response = reqwest::get(url).await?.error_for_status()?;

    let parquet_data = response.bytes().await?;

    // Write to temporary file
    let mut file = tempfile()?;
    file.write_all(&parquet_data)?;

    // Read parquet file using ParquetReader
    let df = ParquetReader::new(&mut file).finish()?;

    // Extract columns
    let dhid_col = df.column("dhid")?.str()?;
    let parent_col = df.column("parent")?.str()?;
    let name_col = df.column("name")?.str()?;
    let lat_col = df.column("lat")?.f32()?;
    let lon_col = df.column("lon")?.f32()?;

    // Convert to DBStation structs
    let mut stations = Vec::new();
    for i in 0..df.height() {
        let dhid = dhid_col.get(i).unwrap_or("").to_string();
        let parent = parent_col.get(i).map(|s| s.to_string());
        let name = name_col.get(i).unwrap_or("").to_string();
        let lat = lat_col.get(i).unwrap_or(0.0) as f64;
        let lon = lon_col.get(i).unwrap_or(0.0) as f64;

        let station = Station {
            dhid,
            parent,
            name,
            lat,
            lon,
        };

        stations.push(DBStation::from_station(station));
    }

    let mut tx = pool.begin().await?;
    clean(&mut tx).await?;
    for transportation in stations {
        if transportation.name.is_empty() {
            continue;
        }
        transportation.store(&mut tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn clean(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM transportation_stations WHERE 1=1")
        .execute(&mut **tx)
        .await
}
