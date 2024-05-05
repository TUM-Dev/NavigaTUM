-- Add up migration script here
-- was never used
DROP TABLE rooms;

-- migrating to using the json type instead of having elaborate insertion logic
alter table de alter column data type json using data::json;

alter table de drop column lat;
alter table de add column lat FLOAT NOT NULL GENERATED ALWAYS AS (CAST (data->'coords'->>'lat' AS FLOAT)) STORED;
alter table de drop column lon;
alter table de add column lon FLOAT NOT NULL GENERATED ALWAYS AS (CAST (data->'coords'->>'lon' AS FLOAT)) STORED;
alter table de drop column name;
alter table de add column name TEXT NOT NULL GENERATED ALWAYS AS (CAST (data->>'name' AS TEXT)) STORED;
alter table de drop column type_common_name;
alter table de add column type_common_name TEXT NOT NULL GENERATED ALWAYS AS (CAST (data->>'type_common_name' AS TEXT)) STORED;
alter table de drop column type;
alter table de add column type TEXT NOT NULL GENERATED ALWAYS AS (CAST (data->>'type' AS TEXT)) STORED;
alter table de drop column calendar_url;
alter table de add column calendar_url TEXT GENERATED ALWAYS AS (CAST (data->'props'->>'calendar_url' AS TEXT)) STORED;

