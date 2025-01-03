use sqlx::PgPool;
use std::time::Duration;

const SECONDS_PER_HOUR: u64 = 60 * 60;
#[tracing::instrument(skip(pool))]
pub async fn all_entries(pool: &PgPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(SECONDS_PER_HOUR)); //
    loop {
        if let Ok(()) = repopulate_indoor_features(pool).await {
            interval.tick().await;
        }
    }
}

#[tracing::instrument(skip(pool))]
async fn repopulate_indoor_features(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query(r#"
    with max_version(max_import_version) as (SELECT MAX(import_version) from indoor_features i2),
         groups_with_outdated_version(group_id, import_version) as (SELECT group_id, import_version
                                                                from indoor_features,
                                                                     max_version
                                                                where import_version < max_import_version)

    DELETE
    FROM indoor_features
    where group_id in (select group_id from groups_with_outdated_version)
      and import_version in (select distinct import_version from groups_with_outdated_version);"#)
        .execute(pool)
        .await?;
    sqlx::query(r#"
    WITH max_version(max_import_version) AS (SELECT MAX(import_version) FROM indoor_features i2),
     geometry(gid, geom, tags) AS (SELECT way_id AS gid, geom, tags
                                   FROM indoor_ways
                                   UNION
                                   DISTINCT
                                   SELECT area_id AS gid, geom, tags
                                   FROM indoor_polygons
                                   UNION
                                   DISTINCT
                                   SELECT node_id AS gid, geom, tags
                                   FROM indoor_nodes),
     geometry_in_lat_lon(gid, geom, tags) AS (SELECT gid, ST_Transform(geom, 4326), tags FROM geometry),
     -- clustered to within about ~20m of non-overlapping distance
     clustered_geometry(gid, group_id, geom, tags)
         AS (SELECT gid,
                    ST_ClusterWithinWin(geom, 0.0001) OVER () AS group_id,
                    geom,
                    tags
             FROM geometry_in_lat_lon),
     clustered_features(group_id, features) AS (SELECT group_id,
                                                       jsonb_build_object(
                                                               'type', 'Feature',
                                                               'id', gid,
                                                               'geometry', ST_AsGeoJSON(geom)::jsonb,
                                                               'properties', tags
                                                       ),
                                                       geom
                                                FROM clustered_geometry),
     grouped_features(group_id, features, convex_hull) AS (SELECT group_id,
                                                                  jsonb_agg(features),
                                                                  ST_ConvexHull(ST_Collect(array_agg(geom)))::geometry
                                                           FROM clustered_features
                                                           GROUP BY group_id
                                                           ORDER BY group_id)

INSERT
INTO indoor_features(group_id, features, convex_hull, import_version)
SELECT group_id, jsonb_build_object('type', 'FeatureCollection','features', features), convex_hull, COALESCE(max_import_version,-1) + 1
FROM grouped_features, max_version
"#).execute(pool).await?;
    Ok(())
}
