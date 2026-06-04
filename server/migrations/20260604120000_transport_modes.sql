DROP TABLE IF EXISTS transportation_stations CASCADE;

CREATE TABLE transportation_stations
(
    id         TEXT PRIMARY KEY,
    name       TEXT   NOT NULL,
    modes      TEXT[] NOT NULL DEFAULT '{}',
    coordinate Point  NOT NULL
);

CREATE INDEX transportation_stations_loc_idx
    ON transportation_stations
        USING GIST (coordinate);
