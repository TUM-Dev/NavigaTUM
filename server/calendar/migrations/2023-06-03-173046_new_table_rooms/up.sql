-- Your SQL goes here
CREATE TABLE IF NOT EXISTS rooms (
    key                   TEXT PRIMARY KEY NOT NULL,
    tumonline_org_id      INTEGER NOT NULL,
    tumonline_calendar_id INTEGER NOT NULL,
    tumonline_room_id     INTEGER NOT NULL,
    last_scrape           TIMESTAMP NOT NULL
);
