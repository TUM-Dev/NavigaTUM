-- CC-BY licensing forces every event image to carry an author credit. The submitter
-- already provides one (stored in img-sources.yaml), but it never reached the popup
-- until now. Adding the column to the events table and the events_active view is what
-- propagates that author into the Martin vector tile feed.

ALTER TABLE events
    ADD COLUMN image_author TEXT NOT NULL DEFAULT '';

-- Re-issue the view to include image_author. CREATE OR REPLACE VIEW only permits
-- appending new columns at the end of the existing SELECT list, so image_author goes
-- after geometry to keep every prior column at its original ordinal.
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
       e.image_author
FROM events e
JOIN tumonline_orgs o ON o.org_id = e.organising_org_id
WHERE now() <= e.ends_at
  AND now() >= e.starts_at
    - (INTERVAL '2 hours'
       * GREATEST(1, CEIL(EXTRACT(EPOCH FROM (e.ends_at - e.starts_at)) / 86400.0)::int));
