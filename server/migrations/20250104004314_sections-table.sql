-- Add migration script here

create materialized view buildings_section as
with avaliable as (select key,
                          jsonb_array_elements(de.data -> 'sections' -> 'buildings_overview' -> 'entries') as entry,
                          (de.data -> 'sections' -> 'buildings_overview' ->> 'n_visible')::integer         as n_visible
                   from de),
     data as (select key,
                     entry ->> 'id'                                              as id,
                     entry ->> 'name'                                            as name,
                     entry ->> 'thumb'                                           as thumb,
                     entry ->> 'subtext'                                         as subtext,
                     substring(entry ->> 'subtext', '([0-9]+) Räume')::integer   as room_cnt,
                     substring(entry ->> 'subtext', '([0-9]+) Gebäude')::integer as building_cnt,
                     n_visible
              from avaliable)

select key,
       id,
       name,
       thumb,
       subtext,
       --building_cnt,room_cnt,
       row_number() over (partition by key order by building_cnt desc ,room_cnt desc) < n_visible as visible
from data
order by key, building_cnt desc, room_cnt desc;

create materialized view rooms_section as
with avaliable as (select key,
                          jsonb_array_elements(de.data -> 'sections' -> 'rooms_overview' -> 'usages') as usage
                   from de),
     data as (select key,
                     usage ->> 'name'                          as name,
                     --(usage->>'count')::integer as count,
                     jsonb_array_elements(usage -> 'children') as child
              from avaliable)

select key,
       data.name        as usage_name,
       child ->> 'id'   as location_id,
       child ->> 'name' as location_name
from data
order by key,
         data.name,
         child ->> 'id';

CREATE INDEX IF NOT EXISTS buildings_section_id_idx ON buildings_section (key);
CREATE INDEX IF NOT EXISTS rooms_section_id_idx ON rooms_section (key);
