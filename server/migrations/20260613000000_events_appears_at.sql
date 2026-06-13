-- Precomputed traffic-weighted lead-in (data/processors/events_appears_at.py); server-side gate only.
-- Backfill existing rows to starts_at before enforcing NOT NULL; the loader reloads real values.
ALTER TABLE events
    ADD COLUMN appears_at TIMESTAMPTZ;
UPDATE events
SET appears_at = starts_at
WHERE appears_at IS NULL;
ALTER TABLE events
    ALTER COLUMN appears_at SET NOT NULL;
ALTER TABLE events
    ADD CONSTRAINT events_appears_at_window
        CHECK (appears_at <= starts_at AND appears_at >= starts_at - INTERVAL '48 hours');

-- Unchanged projection (CREATE OR REPLACE only appends columns), so appears_at stays out of the tiles.
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
       ST_SetSRID(ST_MakePoint(e.coordinate[1], e.coordinate[0]), 4326)::geometry(Point, 4326) AS geometry,
       e.image_author,
       EXTRACT(EPOCH FROM e.starts_at)::bigint AS starts_at_epoch,
       EXTRACT(EPOCH FROM e.ends_at)::bigint   AS ends_at_epoch
FROM events e
JOIN tumonline_orgs o ON o.org_id = e.organising_org_id
WHERE now() BETWEEN e.appears_at AND e.ends_at;

-- The /map "next 2 weeks" feed; superset of events_active (48h cap << 14 days), same appears_at-free projection.
CREATE VIEW events_upcoming AS
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
       ST_SetSRID(ST_MakePoint(e.coordinate[1], e.coordinate[0]), 4326)::geometry(Point, 4326) AS geometry,
       e.image_author,
       EXTRACT(EPOCH FROM e.starts_at)::bigint AS starts_at_epoch,
       EXTRACT(EPOCH FROM e.ends_at)::bigint   AS ends_at_epoch
FROM events e
JOIN tumonline_orgs o ON o.org_id = e.organising_org_id
WHERE now() <= e.ends_at
  AND e.starts_at <= now() + INTERVAL '14 days';
