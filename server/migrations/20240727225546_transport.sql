-- Add migration script here
CREATE TABLE transportation_stations
(
    parent TEXT                    NULL,
    id     TEXT UNIQUE PRIMARY KEY NOT NULL,
    name           TEXT                    NOT NULL,
    coordinate     Point                   NOT NULL
);
ALTER TABLE transportation_stations
    ADD CONSTRAINT transportation_stations_transportation_stations_station_id_fk
        FOREIGN KEY (parent) REFERENCES transportation_stations
            ON UPDATE CASCADE ON DELETE SET NULL;
CREATE INDEX transportation_stations_loc_idx
    ON transportation_stations
        USING GIST (coordinate);
