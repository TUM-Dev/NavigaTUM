-- Add down migration script here
alter table de drop column last_calendar_scrape_at;
alter table en drop column last_calendar_scrape_at;
