-- Add migration script here
-- these are pretty major changes => we should re-download everything
UPDATE en SET last_calendar_scrape_at = NULL WHERE last_calendar_scrape_at IS NOT NULL;
UPDATE de SET last_calendar_scrape_at = NULL WHERE last_calendar_scrape_at IS NOT NULL;
DELETE FROM calendar WHERE 1=1;

ALTER TABLE calendar RENAME COLUMN stp_title_de TO title_de;
ALTER TABLE calendar RENAME COLUMN stp_title_en TO title_en;

ALTER TABLE calendar ALTER COLUMN stp_type DROP NOT NULL;
