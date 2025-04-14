-- Add migration script here

alter table de
    add coordinate_accuracy text generated always as ((((data -> 'coords'::text) ->> 'accuracy'::text))::text) stored null;
alter table en
    add coordinate_accuracy text generated always as ((((data -> 'coords'::text) ->> 'accuracy'::text))::text) stored null;
alter table de
    add coordinate_source text generated always as ((((data -> 'coords'::text) ->> 'source'::text))::text) stored not null;
alter table en
    add coordinate_source text generated always as ((((data -> 'coords'::text) ->> 'source'::text))::text) stored not null;
alter table de
    add comment text generated always as (((data -> 'props'::text) ->> 'comment'::text)::text) stored null;
alter table en
    add comment text generated always as (((data -> 'props'::text) ->> 'comment'::text)::text) stored null;

CREATE MATERIALIZED VIEW ranking_factors AS
SELECT DISTINCT
    data -> 'id' as id,
    data -> 'ranking_factors' ->> 'rank_type' as rank_type,
    data -> 'ranking_factors' ->> 'rank_combined' as rank_combined,
    data -> 'ranking_factors' ->> 'rank_usage' as rank_usage,
    data -> 'ranking_factors' ->> 'rank_custom' as rank_custom,
    data -> 'ranking_factors' ->> 'rank_boost' as rank_boost
from de;

CREATE MATERIALIZED VIEW operators_de AS
SELECT DISTINCT data -> 'props' -> 'operator' ->> 'id'   as id,
                data -> 'props' -> 'operator' ->> 'url'  as url,
                data -> 'props' -> 'operator' ->> 'code' as code,
                data -> 'props' -> 'operator' ->> 'name' as name
from de;

CREATE MATERIALIZED VIEW operators_en AS
SELECT DISTINCT data -> 'props' -> 'operator' ->> 'id'   as id,
                data -> 'props' -> 'operator' ->> 'url'  as url,
                data -> 'props' -> 'operator' ->> 'code' as code,
                data -> 'props' -> 'operator' ->> 'name' as name
from en;

CREATE MATERIALIZED VIEW usage AS
SELECT DISTINCT data -> 'usage' ->> 'name'         as name,
                data -> 'usage' ->> 'din_277'      as din_277,
                data -> 'usage' ->> 'din_277_desc' as din_277_desc
from de
UNION
DISTINCT
SELECT DISTINCT data -> 'usage' ->> 'name'         as name,
                data -> 'usage' ->> 'din_277'      as din_277,
                data -> 'usage' ->> 'din_277_desc' as din_277_desc
from en;

CREATE MATERIALIZED VIEW computed_properties as
(
with facts(key, fact) as (SELECT key, JSON_ARRAY_ELEMENTS((data -> 'props' -> 'computed')::json) as fact
                          from de),
     extracted_facts(key, name, value) as (Select key, fact ->> 'name' as name, fact ->> 'text' as value
                                           From facts)

select distinct f.key,
                room_keys.value    as room_key,
                address.value       as address,
                level.value        as level,
                arch_name.value    as arch_name,
                room_cnt.value     as room_cnt,
                building_cnt.value as building_cnt
from extracted_facts f
         left outer join extracted_facts room_keys on f.key = room_keys.key and room_keys.name = 'Gebäudekennungen'
         left outer join extracted_facts address on f.key = address.key and address.name = 'Adresse'
         left outer join extracted_facts level on f.key = level.key and level.name = 'Stockwerk'
         left outer join extracted_facts arch_name on f.key = arch_name.key and arch_name.name = 'Architekten-Name'
         left outer join extracted_facts room_cnt on f.key = room_cnt.key and room_cnt.name = 'Anzahl Räume'
         left outer join extracted_facts building_cnt
                         on f.key = building_cnt.key and building_cnt.name = 'Anzahl Gebäude'
    );

CREATE MATERIALIZED VIEW urls_de as
(
with unrolled_urls(key, url) as (SELECT key, JSON_ARRAY_ELEMENTS((data -> 'props' ->> 'links')::json) as url
                                 from de)
SELECT key, url ->> 'url' as url, url ->> 'text' as text
FROM unrolled_urls);

CREATE MATERIALIZED VIEW urls_en as
(
with unrolled_urls(key, url) as (SELECT key, JSON_ARRAY_ELEMENTS((data -> 'props' ->> 'links')::json) as url
                                 from en)
SELECT key, url ->> 'url' as url, url ->> 'text' as text
FROM unrolled_urls);

CREATE MATERIALIZED VIEW sources as
(
with unrolled_sources(key, source) as (SELECT key,
                                              JSON_ARRAY_ELEMENTS((data -> 'sources' -> 'base')::json) as source
                                       from de)
SELECT key,
       source ->> 'url'  as url,
       source ->> 'name' as name
FROM unrolled_sources
ORDER BY key, source ->> 'name');
