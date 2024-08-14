-- Add up migration script here
CREATE materialized VIEW indoor_features as
with geometry(gid, geom, tags) as (SELECT way_id as gid, geom, tags
                                   from indoor_ways
                                   union
                                   DISTINCT
                                   SELECT area_id as gid, geom, tags
                                   from indoor_polygons
                                   union
                                   DISTINCT
                                   SELECT node_id as gid, geom, tags
                                   from indoor_nodes),
     geometry_in_lat_lon(gid, geom, tags) as (SELECT gid, ST_Transform(geom, 4326), tags from geometry),
     -- clustered to within about ~2m of non-overlapping distance
     clustered_geometry(gid, group_id, geom, tags)
         as (SELECT gid,
                    ST_ClusterWithinWin(geom, 0.00001) OVER () AS group_id,
                    geom,
                    tags
             from geometry_in_lat_lon),
     clustered_features(group_id, features) AS (SELECT group_id,
                                                       jsonb_build_object(
                                                               'type', 'Feature',
                                                               'id', gid,
                                                               'geometry', ST_AsGeoJSON(geom)::jsonb,
                                                               'properties', tags
                                                       ),
                                                       geom
                                                from clustered_geometry),
     grouped_features(group_id, features, convex_hull) as (SELECT group_id,
                                                                  jsonb_agg(features),
                                                                  ST_ConvexHull(ST_Collect(array_agg(geom)))::geometry
                                                           from clustered_features
                                                           group by group_id
                                                           order by group_id)
SELECT group_id, features, convex_hull
from grouped_features;

CREATE index indoor_features_hull_idx ON indoor_features USING GIST (convex_hull);
CREATE unique index indoor_features_group_idx ON indoor_features(group_id);
