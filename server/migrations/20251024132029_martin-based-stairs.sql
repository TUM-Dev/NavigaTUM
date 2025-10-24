-- Add migration script here

CREATE OR REPLACE
    FUNCTION indoor_stairs(z integer, x integer, y integer, query_params json)
    RETURNS bytea AS $$
DECLARE
  mvt bytea;
BEGIN
  SELECT INTO mvt ST_AsMVT(tile, 'indoor_stairs', 4096, 'geom')
  FROM (
    SELECT
      ST_AsMVTGeom(
          geom,
          ST_TileEnvelope(z, x, y),
          4096, 64, true) AS geom,
      width_cm,
      conveying
    FROM stairs
    WHERE geom && ST_TileEnvelope(z, x, y) AND
          level_min <= COALESCE((query_params->>'level')::real, 0.0) AND
          level_max >= COALESCE((query_params->>'level')::real, 0.0)
  ) as tile
  WHERE geom IS NOT NULL;

  RETURN mvt;
END
$$ LANGUAGE plpgsql IMMUTABLE STRICT PARALLEL SAFE;

DO $do$ BEGIN
    EXECUTE 'COMMENT ON FUNCTION indoor_stairs IS $tj$' || $$
    {
        "description": "indoor rooms",
        "attribution": "\u003Ca href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\"\u003E&copy; OpenStreetMap contributors\u003C/a\u003E",
        "vector_layers": [
            {
                "id": "indoor_stairs",
                "fields": {
                    "width_cm": "Real",
                    "conveying": "Boolean"
                },
                "maxzoom": 30,
                "minzoom": 16
            }
        ]
    }
    $$::json || '$tj$';
END $do$;
