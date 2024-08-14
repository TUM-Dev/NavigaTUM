-- Your SQL goes here
CREATE TABLE calendar (
    key                     VARCHAR(30) NOT NULL,
    dtstart                 TIMESTAMP NOT NULL,
    dtend                   TIMESTAMP NOT NULL,
    dtstamp                 TIMESTAMP NOT NULL,
    event_id                INTEGER NOT NULL,
    event_title             TEXT NOT NULL,
    single_event_id         INTEGER UNIQUE PRIMARY KEY NOT NULL,
    single_event_type_id    TEXT NOT NULL,
    single_event_type_name  TEXT NOT NULL,
    event_type_id           TEXT NOT NULL,
    event_type_name         TEXT,
    course_type_name        TEXT,
    course_type             TEXT,
    course_code             TEXT,
    course_semester_hours   INTEGER,
    group_id                TEXT,
    xgroup                  TEXT,
    status_id               TEXT NOT NULL,
    status                  TEXT NOT NULL,
    comment                 TEXT NOT NULL,
    last_scrape             TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS calendar_lut ON calendar(key, dtstart, dtend);
