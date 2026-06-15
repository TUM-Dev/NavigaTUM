-- 20260614191236's COALESCE fallback 'GEOMETRYCOLLECTION EMPTY' is SRID 0, but the wall geometry
-- is SRID 3857, so ST_Difference rejects door-less tiles with a mixed-SRID error. Match the SRID.

CREATE OR REPLACE
    FUNCTION indoor_walls(z integer, x integer, y integer, query_params json)
    RETURNS bytea AS $$
DECLARE
  mvt bytea;
BEGIN
  SELECT INTO mvt ST_AsMVT(tile, 'indoor_walls', 4096, 'geom')
  FROM (
    SELECT
      ST_AsMVTGeom(
        ST_Difference(
          v_walls.geom,
          COALESCE(v_doors.geom, ST_GeomFromText('GEOMETRYCOLLECTION EMPTY', 3857))
        ),
        ST_TileEnvelope(z, x, y),
        4096, 64, true
      ) AS geom
    FROM (
      SELECT ST_Union(ST_Boundary(ST_CurveToLine(geom))) AS geom
      FROM rooms
      WHERE geom && ST_TileEnvelope(z, x, y)
        AND level_min <= COALESCE((query_params->>'level')::real, 0.0)
        AND level_max >= COALESCE((query_params->>'level')::real, 0.0)
    ) v_walls,
    (
      -- +5cm because we have rounded corners in the rendering and otherwise this looks weird
      SELECT ST_Union(ST_Buffer(geom, width_cm / 100.0 / 2.0 + 0.05, 'endcap=round')) AS geom
      FROM doors
      WHERE geom && ST_TileEnvelope(z, x, y)
            AND level_min <= COALESCE((query_params->>'level')::real, 0.0)
            AND level_max >= COALESCE((query_params->>'level')::real, 0.0)
    ) v_doors
  ) AS tile
  WHERE geom IS NOT NULL;

  RETURN mvt;
END
$$ LANGUAGE plpgsql IMMUTABLE STRICT PARALLEL SAFE;

DO $do$ BEGIN
    EXECUTE 'COMMENT ON FUNCTION indoor_walls IS $tj$' || $$
    {
        "description": "indoor walls",
        "attribution": "<a href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\">&copy; OpenStreetMap contributors</a>",
        "vector_layers": [
            {
                "id": "indoor_walls",
                "fields": {},
                "maxzoom": 30,
                "minzoom": 16
            }
        ]
    }
    $$::json || '$tj$';
END $do$;
