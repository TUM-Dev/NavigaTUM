-- Add down migration script here
ALTER TABLE de DROP COLUMN last_calendar_scrape_at;
ALTER TABLE en DROP COLUMN last_calendar_scrape_at;
