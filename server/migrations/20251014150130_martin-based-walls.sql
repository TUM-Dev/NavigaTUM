-- Add migration script here

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
          v_doors.geom
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
      SELECT ST_Union(ST_Buffer(geom, width_cm / 100.0 / 2.0, 'endcap=round')) AS geom
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
        "attribution": "\u003Ca href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\"\u003E&copy; OpenStreetMap contributors\u003C/a\u003E",
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
