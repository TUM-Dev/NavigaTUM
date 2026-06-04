-- Legacy view used `jsonb_array_elements(...) ->> 0` (NULL on JSON
-- strings); new table holds (entry key, parent id, parent German name).

DROP MATERIALIZED VIEW parents;

CREATE TABLE parents (
    key  TEXT NOT NULL REFERENCES de(key) ON DELETE CASCADE,
    id   TEXT,
    name TEXT
);

CREATE INDEX parents_key_idx ON parents (key);
