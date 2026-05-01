CREATE TABLE events
(
    id             BIGSERIAL PRIMARY KEY,
    name           TEXT        NOT NULL,
    description    TEXT        NULL,
    image          TEXT        NULL,
    coordinate     Point       NOT NULL,
    starts_at      TIMESTAMPTZ NOT NULL,
    ends_at        TIMESTAMPTZ NOT NULL,
    organising_org TEXT        NOT NULL,
    CHECK (ends_at >= starts_at)
);
CREATE INDEX events_loc_idx
    ON events
        USING GIST (coordinate);
CREATE INDEX events_time_idx
    ON events (starts_at, ends_at);
