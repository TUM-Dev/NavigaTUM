-- Exercises the indoor tile functions over whatever the fixture clip ingested, and fails the
-- run (psql is invoked with ON_ERROR_STOP=1) if any of them throws -- e.g. the SRID mismatch
-- that took down the CDN on 2026-06-15. Tiles are derived from the data, not hardcoded, so the
-- gate keeps working as the fixture clip evolves.
--
-- It also asserts the specific regression shape directly: a tile with walls but no doors must
-- still return non-empty wall geometry. That catches BOTH failure modes of indoor_walls --
-- the SRID exception (raises), and the pre-fix blanking where door-less tiles silently lost
-- their walls (empty result).

DO $$
DECLARE
  z       int := 18;
  r       record;
  walls   bytea;
  doorless_walls_seen boolean := false;
BEGIN
  -- Web-mercator (SRID 3857) -> XYZ tile indices for every vertex of every level-0 room.
  FOR r IN
    SELECT DISTINCT
      floor((ST_X(c) + 20037508.342789244) / (2 * 20037508.342789244) * (2 ^ z))::int AS x,
      floor((20037508.342789244 - ST_Y(c)) / (2 * 20037508.342789244) * (2 ^ z))::int AS y
    FROM (
      SELECT (ST_DumpPoints(geom)).geom AS c
      FROM rooms
      WHERE level_min <= 0.0 AND level_max >= 0.0
    ) pts
  LOOP
    -- Any of these raising aborts the whole script (ON_ERROR_STOP), failing the gate.
    walls := indoor_walls(z, r.x, r.y, '{"level":0.0}'::json);
    PERFORM indoor_rooms(z, r.x, r.y, '{"level":0.0}'::json);
    PERFORM indoor_doors(z, r.x, r.y, '{"level":0.0}'::json);
    PERFORM indoor_pois(z, r.x, r.y, '{"level":0.0}'::json);

    IF octet_length(COALESCE(walls, ''::bytea)) > 0
       AND EXISTS (SELECT 1 FROM rooms
                   WHERE level_min <= 0.0 AND level_max >= 0.0
                     AND geom && ST_TileEnvelope(z, r.x, r.y))
       AND NOT EXISTS (SELECT 1 FROM doors
                       WHERE level_min <= 0.0 AND level_max >= 0.0
                         AND geom && ST_TileEnvelope(z, r.x, r.y))
    THEN
      doorless_walls_seen := true;
    END IF;
  END LOOP;

  IF NOT doorless_walls_seen THEN
    RAISE EXCEPTION
      'fixture exercised no walls-without-doors tile at level 0; the regression case is not covered';
  END IF;

  RAISE NOTICE 'indoor tile functions OK across all level-0 room tiles; door-less wall tile verified';
END $$;
