CREATE TABLE events
(
    id                BIGSERIAL PRIMARY KEY,
    name              TEXT        NOT NULL,
    description       TEXT        NULL,
    image             TEXT        NULL,
    coordinate        Point       NOT NULL,
    starts_at         TIMESTAMPTZ NOT NULL,
    ends_at           TIMESTAMPTZ NOT NULL,
    organising_org_id INTEGER     NOT NULL REFERENCES tumonline_orgs (org_id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CHECK (ends_at >= starts_at)
);
CREATE INDEX events_loc_idx
    ON events
        USING GIST (coordinate);
CREATE INDEX events_time_idx
    ON events (starts_at, ends_at);
CREATE INDEX events_org_idx
    ON events (organising_org_id);
