-- Add migration script here

CREATE MATERIALIZED VIEW floors_en AS
SELECT DISTINCT (data -> 'props' -> 'floors' ->> 'id')::integer               as id,
                data -> 'props' -> 'floors' ->> 'floor'                       as floor,
                data -> 'props' -> 'floors' ->> 'tumonline'                   as tumonline,
                data -> 'props' -> 'floors' ->> 'type'                        as type,
                data -> 'props' -> 'floors' ->> 'name'                        as name--,
                --(data -> 'props' -> 'floors' ->> 'mezzanine_shift')::integer  as mezzanine_shift,
                --(data -> 'props' -> 'floors' ->> 'trivial')::boolean          as trivial
from en;

CREATE MATERIALIZED VIEW floors_de AS
SELECT DISTINCT (data -> 'props' -> 'floors' ->> 'id')::integer               as id,
                data -> 'props' -> 'floors' ->> 'floor'                       as floor,
                data -> 'props' -> 'floors' ->> 'tumonline'                   as tumonline,
                data -> 'props' -> 'floors' ->> 'type'                        as type,
                data -> 'props' -> 'floors' ->> 'name'                        as name--,
                --(data -> 'props' -> 'floors' ->> 'mezzanine_shift')::integer  as mezzanine_shift,
                --(data -> 'props' -> 'floors' ->> 'trivial')::boolean          as trivial
from de;
