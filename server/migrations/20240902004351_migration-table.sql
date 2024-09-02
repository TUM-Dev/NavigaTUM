-- Add migration script here
alter table de add operator integer generated always as ((((data -> 'props'::text) -> 'operator'::text) ->> 'id'::text)::integer) stored not null;
alter table en add operator integer generated always as ((((data -> 'props'::text) -> 'operator'::text) ->> 'id'::text)::integer) stored not null;

CREATE VIEW migration_table_de as (
WITH operators_json(id, json_opp) as (SELECT id,
                                             jsonb_build_object(
                                                     'code', code,
                                                     'name', name,
                                                     'id', id,
                                                     'url', url
                                             )
                                      from operators_de)

SELECT data.key                       as id,
       data.type                      as type,
       data.type_common_name          as type_common_name,
       data.name                      as name,
       data.data -> 'aliases'         as aliases,--array
       data.data ->> 'parents'        as parents,--array
       data.data ->> 'parent_names'   as parent_names,--array
       jsonb_build_object(
               'operator', opperator.json_opp,
               'computed', data.data -> 'props' -> 'computed',
               'links', data.data -> 'props' -> 'links',
               'comment', data.data -> 'props' -> 'comment',
               'calendar_url', data.calendar_url
       )                              as props,-- dict data.data -> 'imgs' as imgs, -- dict
       data.data -> 'ranking_factors' as ranking_factors,
       data.data -> 'sources'         as sources,--dict, needs fk-relationship
       --data.data -> 'redirect_url'    as redirect_url, -- added in the backed
       jsonb_build_object(
               'lat', data.lat,
               'lon', data.lon,
               'source', data.coordinate_source,
               'accuracy', data.coordinate_accuracy
       )                              as coords,--hard legacy, likely removed in the future => no furhter work data.data -> 'maps' as maps,--hard legacy, likely removed in the future => no furhter work
       jsonb_build_object(
               'buildings_overview', data.data -> 'sections' -> 'buildings_overview',
               'rooms_overview', data.data -> 'sections' -> 'rooms_overview',
               'featured_overview', data.data -> 'sections' -> 'featured_overview'
       )                              as sections -- pretty complicated dict
FROM de data,
     operators_json opperator
WHERE data.operator = opperator.id);