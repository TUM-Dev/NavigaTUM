DROP MATERIALIZED VIEW sources;

CREATE TABLE sources (
    key     TEXT NOT NULL REFERENCES de(key) ON DELETE CASCADE,
    url     TEXT,
    name    TEXT,
    patched BOOLEAN
);

CREATE INDEX sources_key_idx ON sources (key);
