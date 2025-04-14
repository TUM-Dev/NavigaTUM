-- Add migration script here
CREATE TABLE de
(
    key               TEXT UNIQUE PRIMARY KEY NOT NULL,
    name              TEXT                    NOT NULL,
    tumonline_room_nr INTEGER, -- used for calendars
    type              TEXT                    NOT NULL,
    type_common_name  TEXT                    NOT NULL,
    lat               FLOAT                   NOT NULL,
    lon               FLOAT                   NOT NULL,
    data              TEXT                    NOT NULL
);
CREATE TABLE en
(
    key               TEXT UNIQUE PRIMARY KEY NOT NULL,
    name              TEXT                    NOT NULL,
    tumonline_room_nr INTEGER, -- used for calendars
    type              TEXT                    NOT NULL,
    type_common_name  TEXT                    NOT NULL,
    lat               FLOAT                   NOT NULL,
    lon               FLOAT                   NOT NULL,
    data              TEXT                    NOT NULL,
    FOREIGN KEY(key) REFERENCES de(key)
);
