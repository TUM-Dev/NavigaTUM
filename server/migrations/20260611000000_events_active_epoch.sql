-- The /map events layer filters markers by a "happening now" / "next 24 hours"
-- time window inside a MapLibre filter expression. ST_AsMVT encodes timestamptz
-- properties as text in the session's timezone (Europe/Berlin in production), which
-- a style expression cannot compare robustly across offsets and DST transitions.
-- Epoch seconds give the client a format-stable number to compare against.

-- Re-issue the view to include the epoch columns. CREATE OR REPLACE VIEW only
-- permits appending new columns at the end of the existing SELECT list, so they go
-- after image_author to keep every prior column at its original ordinal.
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
WHERE now() <= e.ends_at
  AND now() >= e.starts_at
    - (INTERVAL '2 hours'
       * GREATEST(1, CEIL(EXTRACT(EPOCH FROM (e.ends_at - e.starts_at)) / 86400.0)::int));
