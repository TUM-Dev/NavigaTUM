use std::env;

use crate::setup::file_loader;
use bytes::Bytes;
use parquet::file::reader::{FileReader as _, SerializedFileReader};
use parquet::record::Field;
use serde::Deserialize;
use sqlx::postgres::PgQueryResult;
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Deserialize, Default, Debug)]
struct Station {
    id: String,
    name: String,
    modes: Vec<String>,
    lat: f64,
    lon: f64,
}

impl Station {
    async fn store(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO transportation_stations(id,name,modes,coordinate)\
            VALUES ($1,$2,$3,POINT($4,$5))",
            self.id,
            self.name,
            &self.modes,
            self.lat,
            self.lon
        )
        .execute(&mut **tx)
        .await
    }
}

#[tracing::instrument(skip(pool))]
pub async fn setup(pool: &PgPool) -> anyhow::Result<()> {
    let cdn_url = env::var("CDN_URL").unwrap_or_else(|_| "https://nav.tum.de/cdn".to_string());
    let body = file_loader::load_file_or_download("public_transport.parquet", &cdn_url).await?;

    let reader = SerializedFileReader::new(Bytes::from(body))?;
    let mut stations = Vec::new();
    for row in reader.get_row_iter(None)? {
        let row = row?;
        let mut station = Station::default();
        for (col_name, field) in row.get_column_iter() {
            match (col_name.as_str(), field) {
                ("id", Field::Str(v)) => station.id.clone_from(v),
                ("name", Field::Str(v)) => station.name.clone_from(v),
                ("modes", Field::ListInternal(list)) => {
                    station.modes = list
                        .elements()
                        .iter()
                        .filter_map(|f| match f {
                            Field::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect();
                }
                ("lat", Field::Float(v)) => station.lat = f64::from(*v),
                ("lat", Field::Double(v)) => station.lat = *v,
                ("lon", Field::Float(v)) => station.lon = f64::from(*v),
                ("lon", Field::Double(v)) => station.lon = *v,
                _ => {}
            }
        }
        stations.push(station);
    }

    let mut tx = pool.begin().await?;
    clean(&mut tx).await?;
    for station in stations {
        if station.name.is_empty() || station.id.is_empty() {
            continue;
        }
        station.store(&mut tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn clean(tx: &mut Transaction<'_, Postgres>) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!("DELETE FROM transportation_stations WHERE 1=1")
        .execute(&mut **tx)
        .await
}
