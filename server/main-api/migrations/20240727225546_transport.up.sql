-- Add up migration script here
CREATE TABLE transportation_stations
(
    parent TEXT                    NULL,
    id     TEXT UNIQUE PRIMARY KEY NOT NULL,
    name           TEXT                    NOT NULL,
    coordinate     Point                   NOT NULL
);
alter table transportation_stations
    add constraint transportation_stations_transportation_stations_station_id_fk
        foreign key (parent) references transportation_stations
            on update cascade on delete set null;
CREATE INDEX transportation_stations_loc_idx
    ON transportation_stations
        USING GIST (coordinate);
