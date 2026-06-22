//! Coordinate overrides sourced from OpenStreetMap.
//!
//! Once a room is mapped in OSM and tagged `ref:tum`, its polygon is the authoritative
//! location. osm2pgsql materialises those polygons into the `rooms` table (same database,
//! separate ownership); here we project each onto a point and rewrite the matching
//! location's `coords`.

use tracing::{debug, info};

/// Per-language override; `{TABLE}` is substituted with a trusted, hard-coded table name.
///
/// We rewrite `data->'coords'` rather than the columns directly, because `lat`, `lon`,
/// `coordinate_source`, and `coordinate_accuracy` are `GENERATED` from it. The point lands
/// on the polygon (`ST_PointOnSurface`, inside even concave rooms), reprojected from web
/// mercator to WGS84; `accuracy` is dropped because this is a precise room-level coordinate.
const OVERRIDE_TEMPLATE: &str = r#"
UPDATE {TABLE} AS t
SET data = jsonb_set(jsonb_set(jsonb_set(
        t.data #- '{coords,accuracy}',
        '{coords,lat}',    to_jsonb(c.lat), true),
        '{coords,lon}',    to_jsonb(c.lon), true),
        '{coords,source}', '"osm"'::jsonb, true)
FROM (
    SELECT DISTINCT ON (ref_tum)
           ref_tum                                           AS key,
           ST_X(ST_Transform(ST_PointOnSurface(geom), 4326)) AS lon,
           ST_Y(ST_Transform(ST_PointOnSurface(geom), 4326)) AS lat
    FROM rooms
    WHERE ref_tum IS NOT NULL AND NOT ST_IsEmpty(geom)
    ORDER BY ref_tum, ST_Area(geom) DESC
) c
WHERE c.key = t.key
"#;

/// The per-language location tables `OVERRIDE_TEMPLATE` is applied to.
const LOCALISED_TABLES: [&str; 2] = ["de", "en"];

/// Rewrites `coords` for every location whose key matches a `ref:tum` tagged room.
///
/// `rooms` is owned by osm2pgsql and absent in migration-only setups (local dev, tests), so
/// it is not part of the schema the `sqlx::query!` macro verifies against: we guard on its
/// existence and use runtime queries. Idempotent, and run on every load so a restart picks
/// up newly tagged rooms and re-applies the override after the pipeline rewrites a `coords`.
#[tracing::instrument(skip(pool))]
pub(super) async fn override_room_coords(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let rooms_table: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.rooms')::text")
            .fetch_one(pool)
            .await?;
    if rooms_table.is_none() {
        debug!("rooms table absent (osm2pgsql not loaded); skipping ref:tum coordinate override");
        return Ok(());
    }

    // If a ref:tum is mapped to more than one polygon, the template prefers the larger one.
    for table in LOCALISED_TABLES {
        // `table` is a hard-coded literal, never user input, so the interpolation is safe.
        let sql = sqlx::AssertSqlSafe(OVERRIDE_TEMPLATE.replace("{TABLE}", table));
        let updated = sqlx::query(sql).execute(pool).await?.rows_affected();
        info!(table, updated, "applied ref:tum coordinate override");
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test fixtures and JSON assertions, consistent with the other setup tests"
)]
mod tests {
    use crate::setup::tests::PostgresTestContainer;
    use rstest::rstest;

    /// `replace` must substitute the `{TABLE}` token only, leaving the JSON-path braces
    /// (`{coords,lat}` …) intact — otherwise the generated SQL is silently corrupted.
    /// The cases mirror `LOCALISED_TABLES`, so every table the override runs against is covered.
    #[rstest]
    #[case::de("de")]
    #[case::en("en")]
    fn substitutes_placeholders_only(#[case] table: &str) {
        assert!(
            super::LOCALISED_TABLES.contains(&table),
            "case `{table}` is not a table the override actually runs against"
        );
        let sql = super::OVERRIDE_TEMPLATE.replace("{TABLE}", table);

        assert!(
            sql.contains(&format!("UPDATE {table} AS t")),
            "table not substituted:{sql}"
        );
        assert!(!sql.contains("{TABLE}"), "placeholder left behind:{sql}");
        for path in [
            "{coords,accuracy}",
            "{coords,lat}",
            "{coords,lon}",
            "{coords,source}",
        ] {
            assert!(sql.contains(path), "json path `{path}` was mangled:{sql}");
        }
    }

    /// Mirrors the osm2pgsql-owned table: generic geometry in web mercator plus `ref:tum`.
    async fn create_rooms_table(pool: &sqlx::Pool<sqlx::Postgres>) {
        sqlx::query("CREATE TABLE rooms (ref_tum text, geom geometry NOT NULL)")
            .execute(pool)
            .await
            .unwrap();
    }

    /// Seeds a `de`+`en` room at (0,0) with a building-accurate non-OSM coordinate, as the
    /// data pipeline would produce it.
    async fn seed_room(pool: &sqlx::Pool<sqlx::Postgres>, key: &str) {
        let data = serde_json::json!({
            "name": key,
            "type": "room",
            "type_common_name": "room",
            "coords": { "lat": 0.0, "lon": 0.0, "source": "inferred", "accuracy": "building" },
        });
        sqlx::query("INSERT INTO de(key, data, hash) VALUES ($1, $2, 0)")
            .bind(key)
            .bind(&data)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO en(key, data) VALUES ($1, $2)")
            .bind(key)
            .bind(&data)
            .execute(pool)
            .await
            .unwrap();
    }

    /// Inserts a square `ref:tum` room of half-width `half` degrees centred on (`lon`, `lat`),
    /// stored in web mercator like osm2pgsql does.
    async fn insert_osm_room(
        pool: &sqlx::Pool<sqlx::Postgres>,
        ref_tum: &str,
        lon: f64,
        lat: f64,
        half: f64,
    ) {
        let wkt = format!(
            "POLYGON(({w} {s},{e} {s},{e} {n},{w} {n},{w} {s}))",
            w = lon - half,
            e = lon + half,
            s = lat - half,
            n = lat + half,
        );
        sqlx::query(
            "INSERT INTO rooms(ref_tum, geom) \
             VALUES ($1, ST_Transform(ST_SetSRID(ST_GeomFromText($2), 4326), 3857))",
        )
        .bind(ref_tum)
        .bind(wkt)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn coords(
        pool: &sqlx::Pool<sqlx::Postgres>,
        query: &'static str,
        key: &str,
    ) -> serde_json::Value {
        sqlx::query_scalar(query)
            .bind(key)
            .fetch_one(pool)
            .await
            .unwrap()
    }
    const DE_COORDS: &str = "SELECT data->'coords' FROM de WHERE key=$1";
    const EN_COORDS: &str = "SELECT data->'coords' FROM en WHERE key=$1";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn no_op_when_rooms_table_absent() {
        let pg = PostgresTestContainer::new().await;
        seed_room(&pg.pool, "0101.01.116").await;

        super::override_room_coords(&pg.pool).await.unwrap();

        let c = coords(&pg.pool, DE_COORDS, "0101.01.116").await;
        assert_eq!(c["source"], "inferred");
        assert_eq!(c["lat"], 0.0);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn overrides_tagged_room_in_both_languages() {
        let pg = PostgresTestContainer::new().await;
        create_rooms_table(&pg.pool).await;
        seed_room(&pg.pool, "0101.01.116").await;
        seed_room(&pg.pool, "0101.01.117").await; // untagged, must stay untouched
        insert_osm_room(&pg.pool, "0101.01.116", 11.6674, 48.2624, 0.0005).await;

        super::override_room_coords(&pg.pool).await.unwrap();

        for query in [DE_COORDS, EN_COORDS] {
            let c = coords(&pg.pool, query, "0101.01.116").await;
            assert_eq!(c["source"], "osm");
            assert!(
                c.get("accuracy").is_none(),
                "OSM coords are room-precise: {c}"
            );
            assert!(
                (c["lat"].as_f64().unwrap() - 48.2624).abs() < 0.001,
                "lat was {c}"
            );
            assert!(
                (c["lon"].as_f64().unwrap() - 11.6674).abs() < 0.001,
                "lon was {c}"
            );
        }

        let untouched = coords(&pg.pool, DE_COORDS, "0101.01.117").await;
        assert_eq!(untouched["source"], "inferred");
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn prefers_larger_polygon_on_duplicate_ref() {
        let pg = PostgresTestContainer::new().await;
        create_rooms_table(&pg.pool).await;
        seed_room(&pg.pool, "0101.01.116").await;
        insert_osm_room(&pg.pool, "0101.01.116", 11.0, 48.0, 0.0001).await; // small
        insert_osm_room(&pg.pool, "0101.01.116", 12.0, 49.0, 0.001).await; // large, wins

        super::override_room_coords(&pg.pool).await.unwrap();

        let c = coords(&pg.pool, DE_COORDS, "0101.01.116").await;
        assert!(
            (c["lat"].as_f64().unwrap() - 49.0).abs() < 0.001,
            "lat was {c}"
        );
        assert!(
            (c["lon"].as_f64().unwrap() - 12.0).abs() < 0.001,
            "lon was {c}"
        );
    }
}
