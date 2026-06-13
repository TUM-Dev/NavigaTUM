-- Student-card validators are a campus-wide overlay, not an indoor layer: they
-- must stay visible regardless of the selected floor. So unlike `indoor_pois`,
-- this function does not filter on `query_params->>'level'`. osm2pgsql captures
-- the OSM `vending_machine`/`student_card_validation` nodes into `pois` with
-- `indoor = 'card_validator'`, carrying their `name` and `ref` (from `ref:tum`).
CREATE OR REPLACE
    FUNCTION card_validators(z integer, x integer, y integer)
    RETURNS bytea AS $$
DECLARE
  mvt bytea;
BEGIN
  SELECT INTO mvt ST_AsMVT(tile, 'card_validators', 4096, 'geom')
  FROM (
    SELECT
      ST_AsMVTGeom(
          geom,
          ST_TileEnvelope(z, x, y),
          4096, 64, true) AS geom,
      name,
      ref
    FROM pois
    WHERE indoor = 'card_validator' AND
          geom && ST_TileEnvelope(z, x, y)
  ) as tile
  WHERE geom IS NOT NULL;

  RETURN mvt;
END
$$ LANGUAGE plpgsql IMMUTABLE STRICT PARALLEL SAFE;

DO $do$ BEGIN
    EXECUTE 'COMMENT ON FUNCTION card_validators IS $tj$' || $$
    {
        "description": "student-card validation machines",
        "attribution": "<a href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\">&copy; OpenStreetMap contributors</a>",
        "vector_layers": [
            {
                "id": "card_validators",
                "fields": {
                    "name": "String",
                    "ref": "String"
                },
                "maxzoom": 30,
                "minzoom": 13
            }
        ]
    }
    $$::json || '$tj$';
END $do$;
