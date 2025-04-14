-- Add migration script here
DROP TABLE IF EXISTS calendar;
DROP INDEX IF EXISTS calendar_lut;

CREATE TYPE EventType AS ENUM ('lecture','exercise','exam','barred','other');
CREATE TABLE calendar
(
    id                  INTEGER UNIQUE PRIMARY KEY NOT NULL,
    room_code           VARCHAR(30)                NOT NULL REFERENCES en,
    start_at            TIMESTAMPTZ                NOT NULL,
    end_at              TIMESTAMPTZ                NOT NULL,
    stp_title_de        TEXT                       NOT NULL,
    stp_title_en        TEXT                       NOT NULL,
    stp_type            TEXT                       NOT NULL,
    entry_type          EventType                  NOT NULL,
    detailed_entry_type TEXT                       NOT NULL
);
CREATE INDEX IF NOT EXISTS calendar_lut ON calendar (room_code, start_at, end_at);
