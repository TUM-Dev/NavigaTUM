use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawStation {
    parent: Option<String>,
    id: String,
    name: String,
    lat: f64,
    lon: f64,
}

pub struct Transportation;

impl Loader for Transportation {
    const FILENAME: &'static str = "public_transport.parquet";
    const TRUNCATE_SQL: &'static str = "TRUNCATE TABLE transportation_stations";
    const ANALYZE_SQL: &'static str = "ANALYZE transportation_stations";
    type Row = RawStation;

    fn parse_field(col: &str, field: &Field, r: &mut Self::Row) {
        match (col, field) {
            // parquet column `dhid` maps to the table's `id` PK.
            ("dhid", Field::Str(v)) => r.id.clone_from(v),
            ("parent", Field::Str(v)) => r.parent = Some(v.clone()),
            ("name", Field::Str(v)) => r.name.clone_from(v),
            ("lat", Field::Float(v)) => r.lat = f64::from(*v),
            ("lat", Field::Double(v)) => r.lat = *v,
            ("lon", Field::Float(v)) => r.lon = f64::from(*v),
            ("lon", Field::Double(v)) => r.lon = *v,
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
        if r.name.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            "INSERT INTO transportation_stations(parent, id, name, coordinate) \
             VALUES ($1, $2, $3, POINT($4, $5))",
            r.parent,
            r.id,
            r.name,
            r.lat,
            r.lon,
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

pub async fn setup(pool: PgPool) -> anyhow::Result<()> {
    super::run::<Transportation>(pool).await
}
