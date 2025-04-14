-- Add migration script here
ALTER TABLE de ADD last_calendar_scrape_at TIMESTAMPTZ DEFAULT NULL;
COMMENT ON COLUMN de.last_calendar_scrape_at IS 'the last time the calendar was scraped for this room';

ALTER TABLE en ADD last_calendar_scrape_at TIMESTAMPTZ DEFAULT NULL;
COMMENT ON COLUMN en.last_calendar_scrape_at IS 'the last time the calendar was scraped for this room';
