-- `is_shower` duplicated `indoor = 'shower'`; `indoor` is the categorisation column.
-- The gender and wheelchair booleans stay: they are typed toilet attributes (sprite
-- choice, popup, /map attribute filters) that no `indoor` value can express.

-- osm2pgsql owns `pois`: replication updates insert through its schema, which would
-- violate the NOT NULL on databases that still carry the column. On fresh databases
-- the import creates the table after migrations run, hence IF EXISTS.
ALTER TABLE IF EXISTS pois
    DROP COLUMN IF EXISTS is_shower;

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
        "attribution": "\u003Ca href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\"\u003E&copy; OpenStreetMap contributors\u003C/a\u003E",
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
