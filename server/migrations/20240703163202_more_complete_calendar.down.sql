-- Add down migration script here

ALTER TABLE calendar RENAME COLUMN title_de TO stp_title_de;
ALTER TABLE calendar RENAME COLUMN title_en TO stp_title_en;

DELETE FROM calendar WHERE stp_type IS NULL;
ALTER TABLE calendar ALTER COLUMN stp_type SET NOT NULL;
