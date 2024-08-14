-- Add down migration script here
-- was never used
CREATE TABLE rooms
(
    key                   TEXT PRIMARY KEY            NOT NULL,
    tumonline_org_id      INTEGER                     NOT NULL,
    tumonline_calendar_id INTEGER                     NOT NULL,
    tumonline_room_id     INTEGER                     NOT NULL,
    last_scrape           TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

-- migrating to
DROP TABLE en;
CREATE TABLE en
(
    key                     TEXT             NOT NULL
        PRIMARY KEY
        REFERENCES de,
    name                    TEXT             NOT NULL,
    tumonline_room_nr       INTEGER,
    type                    TEXT             NOT NULL,
    type_common_name        TEXT             NOT NULL,
    lat                     double precision NOT NULL,
    lon                     double precision NOT NULL,
    data                    TEXT             NOT NULL,
    last_calendar_scrape_at TIMESTAMP WITH TIME ZONE
);
COMMENT ON COLUMN en.last_calendar_scrape_at IS 'the last time the calendar was scraped for this room';

DROP TABLE de;
CREATE TABLE de
(
    key                     TEXT             NOT NULL PRIMARY KEY,
    name                    TEXT             NOT NULL,
    tumonline_room_nr       INTEGER,
    type                    TEXT             NOT NULL,
    type_common_name        TEXT             NOT NULL,
    lat                     double precision NOT NULL,
    lon                     double precision NOT NULL,
    data                    TEXT             NOT NULL,
    last_calendar_scrape_at TIMESTAMP WITH TIME ZONE
);
COMMENT ON COLUMN de.last_calendar_scrape_at IS 'the last time the calendar was scraped for this room';
