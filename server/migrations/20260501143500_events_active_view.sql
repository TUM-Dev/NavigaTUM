-- Currently-active events plus a 2-hour-per-day prebuffer.
-- An event is "active" between (starts_at - 2h * days_long) and ends_at.
-- days_long is ceil((ends_at - starts_at) / 1 day), with a floor of 1 so a
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
       ST_SetSRID(ST_MakePoint(e.coordinate[1], e.coordinate[0]), 4326) AS geometry
FROM events e
JOIN tumonline_orgs o ON o.org_id = e.organising_org_id
WHERE now() <= e.ends_at
  AND now() >= e.starts_at
    - (INTERVAL '2 hours'
       * GREATEST(1, CEIL(EXTRACT(EPOCH FROM (e.ends_at - e.starts_at)) / 86400.0)::int));
