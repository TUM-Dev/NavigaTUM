-- Add migration script here

CREATE OR REPLACE
    FUNCTION indoor_rooms(z integer, x integer, y integer, query_params json)
    RETURNS bytea AS $$
DECLARE
  mvt bytea;
BEGIN
  SELECT INTO mvt ST_AsMVT(tile, 'indoor_rooms', 4096, 'geom')
  FROM (
    SELECT
      ST_AsMVTGeom(
          ST_CurveToLine(geom),
          ST_TileEnvelope(z, x, y),
          4096, 64, true) AS geom,
      indoor,
      ref,
      ref_tum
    FROM rooms
    WHERE geom && ST_TileEnvelope(z, x, y) AND
          level_min <= COALESCE((query_params->>'level')::real, 0.0) AND
          level_max >= COALESCE((query_params->>'level')::real, 0.0)
  ) as tile
  WHERE geom IS NOT NULL;

  RETURN mvt;
END
$$ LANGUAGE plpgsql IMMUTABLE STRICT PARALLEL SAFE;

DO $do$ BEGIN
    EXECUTE 'COMMENT ON FUNCTION indoor_rooms IS $tj$' || $$
    {
        "description": "indoor rooms",
        "attribution": "\u003Ca href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\"\u003E&copy; OpenStreetMap contributors\u003C/a\u003E",
        "vector_layers": [
            {
                "id": "indoor_rooms",
                "fields": {
                    "indoor": "String",
                    "ref": "String",
                    "ref_tum": "String"
                },
                "maxzoom": 30,
                "minzoom": 16
            }
        ]
    }
    $$::json || '$tj$';
END $do$;

CREATE OR REPLACE
    FUNCTION indoor_doors(z integer, x integer, y integer, query_params json)
    RETURNS bytea AS $$
DECLARE
  mvt bytea;
BEGIN
  SELECT INTO mvt ST_AsMVT(tile, 'indoor_doors', 4096, 'geom')
  FROM (
    SELECT
      ST_AsMVTGeom(
          geom,
          ST_TileEnvelope(z, x, y),
          4096, 64, true) AS geom,
      width_cm
    FROM doors
    WHERE geom && ST_TileEnvelope(z, x, y) AND
          level_min <= COALESCE((query_params->>'level')::real, 0.0) AND
          level_max >= COALESCE((query_params->>'level')::real, 0.0)
  ) as tile
  WHERE geom IS NOT NULL;

  RETURN mvt;
END
$$ LANGUAGE plpgsql IMMUTABLE STRICT PARALLEL SAFE;

DO $do$ BEGIN
    EXECUTE 'COMMENT ON FUNCTION indoor_doors IS $tj$' || $$
    {
        "description": "indoor doors",
        "attribution": "\u003Ca href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\"\u003E&copy; OpenStreetMap contributors\u003C/a\u003E",
        "vector_layers": [
            {
                "id": "indoor_doors",
                "fields": {
                    "width_cm": "Number"
                },
                "maxzoom": 30,
                "minzoom": 16
            }
        ]
    }
    $$::json || '$tj$';
END $do$;
