-- Bulk mode renders plain point/line/polygon tables only, so the geometry work the former
-- `indoor_*` tile functions did per request (curve linearisation, door cut-outs) is done once here.
DROP TABLE IF EXISTS indoor_bulk_rooms, indoor_bulk_walls;

CREATE TABLE indoor_bulk_rooms AS
SELECT
    row_number() OVER () AS id,
    indoor,
    ref_tum,
    students_have_access,
    is_male_toilet,
    is_female_toilet,
    is_wheelchair_toilet,
    level_min,
    level_max,
    ST_Multi(ST_CollectionExtract(ST_CurveToLine(geom), 3)) AS geom
FROM rooms
WHERE NOT ST_IsEmpty(geom);

CREATE INDEX ON indoor_bulk_rooms USING gist (geom);

-- One row set per floor a room is on, so a door only cuts an opening on the floors it exists on.
CREATE TABLE indoor_bulk_walls AS
SELECT
    row_number() OVER () AS id,
    walls.level,
    ST_Multi(ST_CollectionExtract(walls.geom, 2)) AS geom
FROM (
    SELECT
        lv.level,
        ST_Difference(
            ST_Boundary(ST_CurveToLine(r.geom)),
            COALESCE(
                (
                    -- +5cm because we have rounded corners in the rendering and otherwise this looks weird
                    SELECT ST_Union(ST_Buffer(d.geom, d.width_cm / 100.0 / 2.0 + 0.05, 'endcap=round'))
                    FROM doors d
                    WHERE ST_DWithin(d.geom, r.geom, d.width_cm / 100.0 / 2.0 + 0.05)
                      AND d.level_min <= lv.level
                      AND d.level_max >= lv.level
                ),
                ST_GeomFromText('GEOMETRYCOLLECTION EMPTY', ST_SRID(r.geom))
            )
        ) AS geom
    FROM rooms r
    CROSS JOIN LATERAL unnest(:'levels'::numeric[]) AS lv(level)
    WHERE NOT ST_IsEmpty(r.geom)
      AND lv.level BETWEEN r.level_min AND r.level_max
) walls
WHERE NOT ST_IsEmpty(walls.geom);

CREATE INDEX ON indoor_bulk_walls USING gist (geom);
ANALYZE indoor_bulk_rooms, indoor_bulk_walls;
