use parquet::record::Field;
use sqlx::{PgPool, Postgres, Transaction};

use super::Loader;

#[derive(Debug, Default)]
pub struct RawStation {
    id: String,
    name: String,
    modes: Vec<String>,
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
            ("id", Field::Str(v)) => r.id.clone_from(v),
            ("name", Field::Str(v)) => r.name.clone_from(v),
            ("modes", Field::ListInternal(list)) => {
                r.modes = list
                    .elements()
                    .iter()
                    .filter_map(|f| match f {
                        Field::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
            }
            ("lat", Field::Float(v)) => r.lat = f64::from(*v),
            ("lat", Field::Double(v)) => r.lat = *v,
            ("lon", Field::Float(v)) => r.lon = f64::from(*v),
            ("lon", Field::Double(v)) => r.lon = *v,
            _ => {}
        }
    }

    async fn insert(tx: &mut Transaction<'_, Postgres>, r: &Self::Row) -> anyhow::Result<()> {
        // The migration enforces NOT NULL on id/name and cardinality(modes) > 0;
        // skip parquet rows that would violate those constraints.
        if r.id.is_empty() || r.name.is_empty() || r.modes.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            "INSERT INTO transportation_stations(id, name, modes, coordinate) \
             VALUES ($1, $2, $3, POINT($4, $5))",
            r.id,
            r.name,
            &r.modes,
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
