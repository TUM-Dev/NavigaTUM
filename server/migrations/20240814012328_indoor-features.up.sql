-- Add up migration script here
CREATE MATERIALIZED VIEW indoor_features AS
with geometry(gid, geom, tags) AS (SELECT way_id AS gid, geom, tags
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
SELECT group_id, features, convex_hull
FROM grouped_features;

CREATE index indoor_features_hull_idx ON indoor_features USING GIST (convex_hull);
CREATE UNIQUE index indoor_features_group_idx ON indoor_features(group_id);
