use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
struct StationInfo {
    station_id: String,
    name: String,
    lat: f64,
    lon: f64,
}

#[derive(Deserialize, Default, Debug)]
struct Station {
    #[serde(flatten)]
    station: StationInfo,
    sub_stations: Vec<StationInfo>,
}

struct DBStation {
    parent: Option<String>,
    id: String,
    name: String,
    lat: f64,
    lon: f64,
}

impl DBStation {
    fn from_station(info: StationInfo, parent: Option<String>) -> Self {
        Self {
            parent,
            id: info.station_id,
            name: info.name,
            lat: info.lat,
            lon: info.lon,
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
    let url = "https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/main/data/external/results/public_transport.json";
    let transportations = reqwest::get(url)
        .await?
        .error_for_status()?
        .json::<Vec<Station>>()
        .await?;
    let transportations = transportations.into_iter().flat_map(|s| {
        let id = s.station.station_id.clone();
        let mut stations = vec![DBStation::from_station(s.station, None)];
        for sub in s.sub_stations.into_iter() {
            stations.push(DBStation::from_station(sub, Some(id.clone())))
        }
        stations
    });
    let mut tx = pool.begin().await?;
    clean(&mut tx).await?;
    for transportation in transportations {
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
