-- Add down migration script here
DROP INDEX aliases_alias_key_uindex;
DROP TABLE IF EXISTS aliases;
