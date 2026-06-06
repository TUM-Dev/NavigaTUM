DROP MATERIALIZED VIEW urls_en;

CREATE TABLE urls_en (
    key  TEXT NOT NULL REFERENCES en(key) ON DELETE CASCADE,
    url  TEXT,
    text TEXT
);

CREATE INDEX urls_en_key_idx ON urls_en (key);
