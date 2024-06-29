-- Add down migration script here
DROP INDEX IF EXISTS hash_lut;
alter table de drop column hash;
