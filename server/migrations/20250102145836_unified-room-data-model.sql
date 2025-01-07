-- Add migration script here

DROP MATERIALIZED VIEW ranking_factors;
CREATE MATERIALIZED VIEW ranking_factors AS
SELECT DISTINCT data ->> 'id'                                            as id,
                (data -> 'ranking_factors' ->> 'rank_type')::integer     as rank_type,
                (data -> 'ranking_factors' ->> 'rank_combined')::integer as rank_combined,
                (data -> 'ranking_factors' ->> 'rank_usage')::integer    as rank_usage,
                (data -> 'ranking_factors' ->> 'rank_custom')::integer   as rank_custom,
                (data -> 'ranking_factors' ->> 'rank_boost')::integer    as rank_boost
from de;

DROP MATERIALIZED VIEW usage;
CREATE MATERIALIZED VIEW usages AS
SELECT DISTINCT hashtext(data -> 'usage' ->> 'name') as usage_id,
                data -> 'usage' ->> 'name'           as name,
                data -> 'usage' ->> 'din_277'        as din_277,
                data -> 'usage' ->> 'din_277_desc'   as din_277_desc
from de
UNION
DISTINCT
SELECT DISTINCT hashtext(data -> 'usage' ->> 'name') as usage_id,
                data -> 'usage' ->> 'name'           as name,
                data -> 'usage' ->> 'din_277'        as din_277,
                data -> 'usage' ->> 'din_277_desc'   as din_277_desc
from en;

ALTER TABLE de
    ADD COLUMN usage_id INTEGER NULL GENERATED ALWAYS AS (hashtext(data -> 'usage' ->> 'name')) STORED;
ALTER TABLE en
    ADD COLUMN usage_id INTEGER NULL GENERATED ALWAYS AS (hashtext(data -> 'usage' ->> 'name')) STORED;
ALTER TABLE de
    ADD COLUMN operator_id INTEGER NULL GENERATED ALWAYS AS ((data -> 'props' -> 'operator' ->> 'id')::integer) STORED;
ALTER TABLE en
    ADD COLUMN operator_id INTEGER NULL GENERATED ALWAYS AS ((data -> 'props' -> 'operator' ->> 'id')::integer) STORED;

DROP MATERIALIZED VIEW sources;
CREATE MATERIALIZED VIEW sources as
WITH unrolled_sources(key, source) as (SELECT key,
                                              JSONB_ARRAY_ELEMENTS(data -> 'sources' -> 'base') as source,
                                              (data -> 'sources' ->> 'patched')::bool           as patched
                                       FROM de)
SELECT key,
       source ->> 'url'  as url,
       source ->> 'name' as name,
       patched
FROM unrolled_sources
ORDER BY key, source ->> 'name';

DROP MATERIALIZED VIEW operators_de;
CREATE MATERIALIZED VIEW operators_de AS
SELECT DISTINCT (data -> 'props' -> 'operator' ->> 'id')::integer as id,
                data -> 'props' -> 'operator' ->> 'url'           as url,
                data -> 'props' -> 'operator' ->> 'code'          as code,
                data -> 'props' -> 'operator' ->> 'name'          as name
from de;

DROP MATERIALIZED VIEW operators_en;
CREATE MATERIALIZED VIEW operators_en AS
SELECT DISTINCT (data -> 'props' -> 'operator' ->> 'id')::integer as id,
                data -> 'props' -> 'operator' ->> 'url'           as url,
                data -> 'props' -> 'operator' ->> 'code'          as code,
                data -> 'props' -> 'operator' ->> 'name'          as name
from en;

drop materialized view computed_properties;
create materialized view computed_properties as
WITH facts(key, fact) AS (SELECT de.key,
                                 jsonb_array_elements((de.data -> 'props') -> 'computed') AS fact
                          FROM de),
     extracted_facts(key, name, value) AS (SELECT facts.key,
                                                  facts.fact ->> 'name' AS name,
                                                  facts.fact ->> 'text' AS value
                                           FROM facts
                                           where facts.fact ->> 'text' != '')
SELECT DISTINCT f.key,
                building_codes.value                                                                 AS building_codes,
                split_part(address.value, ', ', 1)                                                   AS address,
                split_part(split_part(address.value, ', ', 2), ' ', 1)::integer                      AS postcode,
                split_part(split_part(address.value, ', ', 2), ' ', 2)                               AS city,
                level.value                                                                          AS level,
                arch_name.value                                                                      AS arch_name,
                split_part(room_cnt.value, ' (', 1)::integer                                         AS room_cnt,
                (case
                     when room_cnt.value like '%(%'
                         then (split_part(split_part(room_cnt.value, '(', 2), ' ', 1)::integer) end) AS room_cnt_without_corridors,
                building_cnt.value::integer                                                          AS building_cnt
FROM extracted_facts f
         LEFT JOIN extracted_facts building_codes
                   ON f.key = building_codes.key AND building_codes.name = 'Gebäudekennungen'
         LEFT JOIN extracted_facts address ON f.key = address.key AND address.name = 'Adresse'
         LEFT JOIN extracted_facts level ON f.key = level.key AND level.name = 'Stockwerk'
         LEFT JOIN extracted_facts arch_name ON f.key = arch_name.key AND arch_name.name = 'Architekten-Name'
         LEFT JOIN extracted_facts room_cnt ON f.key = room_cnt.key AND room_cnt.name = 'Anzahl Räume'
         LEFT JOIN extracted_facts building_cnt
                   ON f.key = building_cnt.key AND building_cnt.name = 'Anzahl Gebäude';

create materialized view parents as
SELECT key,
       jsonb_array_elements(data -> 'parents') ->> 0      as id,
       jsonb_array_elements(data -> 'parent_names') ->> 0 as name
FROM de;

create materialized view overlay_maps as
with avaliable as (select key, jsonb_array_elements(de.data -> 'maps' -> 'overlays' -> 'available') as available
                   from de),
     data as (SELECT key,
                     (a.available ->> 'id')::integer as id,
                     a.available ->> 'floor'         as floor,
                     a.available ->> 'name'          as name,
                     a.available ->> 'file'          as file,
                     a.available ->> 'coordinates'   as coordinates
              FROM avaliable a),
     coordinates as
         (Select key,
                 id,
                 unnest(regexp_matches(coordinates, '\[([\d]+.[\d]+)', 'g'))::float as lon,
                 unnest(regexp_matches(coordinates, ', ([\d]+.[\d]+)', 'g'))::float as lat
          FROM data),
     default_overlays as (SELECT key,
                                 (data -> 'maps' -> 'overlays' ->> 'default')::integer as default_id
                          from de)

Select data.key,
       data.id                               as id,
       data.floor                            as floor,
       data.name                             as name,
       data.file                             as file,
       array_agg(coordinates.lon)            as coordinates_lon,
       array_agg(coordinates.lat)            as coordinates_lat,
       default_overlays.default_id = data.id as selected_by_default
FROM data,
     coordinates,
     default_overlays
where coordinates.key = data.key
  and default_overlays.key = data.key
  and coordinates.id = data.id
group by data.key, data.id, data.floor, data.name, data.file, default_overlays.default_id;

create materialized view roomfinder_maps as
with avaliable as (select key, jsonb_array_elements(de.data -> 'maps' -> 'roomfinder' -> 'available') as available
                   from de),
     data as (SELECT key,
                     a.available ->> 'name'              as name,
                     a.available ->> 'id'                as id,
                     (a.available ->> 'scale')::integer  as scale,
                     (a.available ->> 'height')::integer as height,
                     (a.available ->> 'width')::integer  as width,
                     (a.available ->> 'x')::integer      as x,
                     (a.available ->> 'y')::integer      as y,
                     a.available ->> 'source'            as source,
                     a.available ->> 'file'              as file
              FROM avaliable a),
     default_overlays as (SELECT key,
                                 data -> 'maps' -> 'roomfinder' ->> 'default' as default_id
                          from de)

Select data.key,
       data.name                             as name,
       data.id                               as id,
       data.scale                            as scale,
       data.height                           as height,
       data.width                            as width,
       data.x                                as x,
       data.y                                as y,
       data.source                           as source,
       data.file                             as file,
       default_overlays.default_id = data.id as selected_by_default
FROM data,
     default_overlays
where default_overlays.key = data.key
group by data.key, data.name, data.id, data.scale, data.height, data.width, data.x, data.y, data.source, data.file,
         default_overlays.default_id;

CREATE INDEX IF NOT EXISTS ranking_factors_id_idx ON ranking_factors (id);
CREATE INDEX IF NOT EXISTS usage_id_idx ON usages (usage_id);
CREATE INDEX IF NOT EXISTS sources_idx ON sources (key);
CREATE INDEX IF NOT EXISTS urls_de_id_idx ON urls_de (key);
CREATE INDEX IF NOT EXISTS urls_en_id_idx ON urls_en (key);
CREATE INDEX IF NOT EXISTS operators_de_id_idx ON operators_de (id);
CREATE INDEX IF NOT EXISTS operators_en_id_idx ON operators_en (id);
CREATE INDEX IF NOT EXISTS computed_properties_id_idx ON computed_properties (key);
CREATE INDEX IF NOT EXISTS parents_id_idx ON parents (key);
CREATE INDEX IF NOT EXISTS roomfinder_maps_id_idx ON roomfinder_maps (key);
CREATE INDEX IF NOT EXISTS overlay_maps_id_idx ON overlay_maps (key);
