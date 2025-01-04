-- Add migration script here

create materialized view location_images as
with avaliable as (select key, jsonb_array_elements(de.data -> 'imgs') as available
                   from de)
SELECT a.key,
       a.available ->> 'name'              as name,
       a.available -> 'author' ->> 'url'   as author_url,
       a.available -> 'author' ->> 'text'  as author_text,
       a.available -> 'source' ->> 'url'   as source_url,
       a.available -> 'source' ->> 'text'  as source_text,
       a.available -> 'license' ->> 'url'  as license_url,
       a.available -> 'license' ->> 'text' as license_text
from avaliable a;

CREATE INDEX IF NOT EXISTS location_images_id_idx ON location_images (key);
