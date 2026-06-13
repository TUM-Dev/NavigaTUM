-- Carry the WC attribute booleans on `rooms` so the room-fill layer can dim
-- non-matching toilet rooms the same way the POI filter hides their icons.
-- The flags already exist on `pois`; mirroring them here avoids a client-side
-- spatial join between rooms and POIs.

-- osm2pgsql owns `rooms`: replication updates insert through its schema, which
-- would violate the NOT NULL on databases that still lack the columns. On fresh
-- databases the import creates the table after migrations run, hence IF EXISTS
-- and column defaults so existing rows backfill to "no flag set".
ALTER TABLE IF EXISTS rooms
    ADD COLUMN IF NOT EXISTS is_male_toilet       boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_female_toilet     boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_wheelchair_toilet boolean NOT NULL DEFAULT false;

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
