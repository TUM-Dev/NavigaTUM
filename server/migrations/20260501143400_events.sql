CREATE TABLE tumonline_orgs
(
    org_id  INTEGER PRIMARY KEY NOT NULL,
    code    TEXT UNIQUE         NOT NULL,
    name_de TEXT                NOT NULL,
    name_en TEXT                NOT NULL,
    path_de TEXT                NULL,
    path_en TEXT                NULL
);
CREATE INDEX tumonline_orgs_code_idx
    ON tumonline_orgs (code);

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

-- Currently-active events plus a 2-hour-per-day prebuffer.
-- An event is "active" between (starts_at - 2h * days_long) and ends_at.
-- days_long = ceil((ends_at - starts_at) / 1 day), with a floor of 1 so a
-- sub-24h event still gets the full 2h prebuffer.
-- Exposed to Martin via the pg-tables source mechanism.
CREATE OR REPLACE VIEW events_active AS
SELECT e.id,
       e.name,
       e.description,
       e.image,
       e.starts_at,
       e.ends_at,
       e.organising_org_id,
       o.code    AS organising_org_code,
       o.name_de AS organising_org_name_de,
       o.name_en AS organising_org_name_en,
       ST_SetSRID(ST_MakePoint(e.coordinate[1], e.coordinate[0]), 4326)::geometry(Point, 4326) AS geometry
FROM events e
JOIN tumonline_orgs o ON o.org_id = e.organising_org_id
WHERE now() <= e.ends_at
  AND now() >= e.starts_at
    - (INTERVAL '2 hours'
       * GREATEST(1, CEIL(EXTRACT(EPOCH FROM (e.ends_at - e.starts_at)) / 86400.0)::int));
