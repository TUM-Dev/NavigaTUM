-- Add down migration script here
DROP INDEX IF EXISTS hash_lut;
ALTER TABLE de DROP COLUMN hash;
