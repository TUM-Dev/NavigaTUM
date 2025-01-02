use sqlx::PgPool;

pub struct Transportation {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub parent_name: Option<String>,
    pub lat: Option<f64>,             // not really null, sqlx just thinks this
    pub lon: Option<f64>,             // not really null, sqlx just thinks this
    pub distance_meters: Option<f64>, // not really null, sqlx just thinks this
}
impl Transportation {
    pub async fn fetch_all_near(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Transportation>> {
        // TODO: use the spatial index instead of just computing the distance for every entry
        sqlx::query_as!(
            Transportation,
            r#"
WITH coodinates_for_keys(key, coordinate) as (SELECT key, point(lat, lon)::geometry as coordinate
                                              from de)

SELECT t.id,
       t.name,
       parent.id as parent_id,
       parent.name as parent_name,
       ST_X(t.coordinate::geometry)                             as lat,
       ST_Y(t.coordinate::geometry)                             as lon,
       ST_DISTANCE(t.coordinate::geometry, c.coordinate, false) as distance_meters
FROM coodinates_for_keys c,
     transportation_stations t
     LEFT OUTER JOIN transportation_stations parent on t.parent = parent.id
WHERE ST_DISTANCE(t.coordinate::geometry, c.coordinate, false) < 1000
  AND c.key = $1
ORDER BY ST_DISTANCE(t.coordinate::geometry, c.coordinate, false)
LIMIT 50"#,
            id
        )
        .fetch_all(pool)
        .await
    }
}
