-- Add up migration script here
alter table de add last_calendar_scrape_at TIMESTAMPTZ default null;
comment on column de.last_calendar_scrape_at is 'the last time the calendar was scraped for this room';

alter table en add last_calendar_scrape_at TIMESTAMPTZ default null;
comment on column en.last_calendar_scrape_at is 'the last time the calendar was scraped for this room';
