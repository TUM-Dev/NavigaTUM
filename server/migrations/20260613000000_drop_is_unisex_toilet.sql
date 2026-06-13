-- A unisex/all-gender toilet is usable by everyone, so it is now encoded as both
-- `is_male_toilet` and `is_female_toilet` rather than a separate `is_unisex_toilet`
-- flag. Backfill existing explicitly-unisex rows into that pair before dropping the
-- column, then re-emit the tiles without it.

-- osm2pgsql owns `pois`/`rooms`: replication updates insert through its schema, which
-- no longer carries the column. On fresh databases the import creates the tables after
-- migrations run, so guard the backfill on the tables already existing.
DO $$ BEGIN
  IF to_regclass('public.pois') IS NOT NULL THEN
    UPDATE pois SET is_male_toilet = true, is_female_toilet = true WHERE is_unisex_toilet;
  END IF;
  IF to_regclass('public.rooms') IS NOT NULL THEN
    UPDATE rooms SET is_male_toilet = true, is_female_toilet = true WHERE is_unisex_toilet;
  END IF;
END $$;

ALTER TABLE IF EXISTS pois  DROP COLUMN IF EXISTS is_unisex_toilet;
ALTER TABLE IF EXISTS rooms DROP COLUMN IF EXISTS is_unisex_toilet;

CREATE OR REPLACE
    FUNCTION indoor_pois(z integer, x integer, y integer, query_params json)
    RETURNS bytea AS $$
DECLARE
  mvt bytea;
BEGIN
  SELECT INTO mvt ST_AsMVT(tile, 'indoor_pois', 4096, 'geom')
  FROM (
    SELECT
      ST_AsMVTGeom(
          geom,
          ST_TileEnvelope(z, x, y),
          4096, 64, true) AS geom,
      indoor,
      name,
      ref,
      students_have_access,
      is_male_toilet,
      is_female_toilet,
      is_wheelchair_toilet,
      area
    FROM pois
    WHERE geom && ST_TileEnvelope(z, x, y) AND
          level_min <= COALESCE((query_params->>'level')::real, 0.0) AND
          level_max >= COALESCE((query_params->>'level')::real, 0.0)
  ) as tile
  WHERE geom IS NOT NULL;

  RETURN mvt;
END
$$ LANGUAGE plpgsql IMMUTABLE STRICT PARALLEL SAFE;

DO $do$ BEGIN
    EXECUTE 'COMMENT ON FUNCTION indoor_pois IS $tj$' || $$
    {
        "description": "indoor rooms",
        "attribution": "<a href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\">&copy; OpenStreetMap contributors</a>",
        "vector_layers": [
            {
                "id": "indoor_pois",
                "fields": {
                    "indoor": "String",
                    "name": "String",
                    "ref": "String",
                    "students_have_access": "Boolean",
                    "is_male_toilet": "Boolean",
                    "is_female_toilet": "Boolean",
                    "is_wheelchair_toilet": "Boolean",
                    "area": "Real"
                },
                "maxzoom": 30,
                "minzoom": 16
            }
        ]
    }
    $$::json || '$tj$';
END $do$;

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
      ref_tum,
      students_have_access,
      is_male_toilet,
      is_female_toilet,
      is_wheelchair_toilet
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
        "attribution": "<a href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\">&copy; OpenStreetMap contributors</a>",
        "vector_layers": [
            {
                "id": "indoor_rooms",
                "fields": {
                    "indoor": "String",
                    "ref_tum": "String",
                    "students_have_access": "Boolean",
                    "is_male_toilet": "Boolean",
                    "is_female_toilet": "Boolean",
                    "is_wheelchair_toilet": "Boolean"
                },
                "maxzoom": 30,
                "minzoom": 16
            }
        ]
    }
    $$::json || '$tj$';
END $do$;
