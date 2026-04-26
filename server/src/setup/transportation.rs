use crate::setup::file_loader;
use bytes::Bytes;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use serde::Deserialize;

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
    let cdn_url = std::env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("public_transport.parquet", &cdn_url).await?;

    let reader = SerializedFileReader::new(Bytes::from(body))?;
    let mut stations = Vec::new();
    for row in reader.get_row_iter(None)? {
        let row = row?;
        let mut station = Station::default();
        for (col_name, field) in row.get_column_iter() {
            match (col_name.as_str(), field) {
                ("dhid", Field::Str(v)) => station.dhid = v.clone(),
                ("parent", Field::Str(v)) => station.parent = Some(v.clone()),
                ("name", Field::Str(v)) => station.name = v.clone(),
                ("lat", Field::Float(v)) => station.lat = f64::from(*v),
                ("lat", Field::Double(v)) => station.lat = *v,
                ("lon", Field::Float(v)) => station.lon = f64::from(*v),
                ("lon", Field::Double(v)) => station.lon = *v,
                _ => {}
            }
        }
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
