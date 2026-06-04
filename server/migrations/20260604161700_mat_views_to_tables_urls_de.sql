-- Legacy view stored each link's bilingual `{de, en}` object as text;
-- new table holds the German values.

DROP MATERIALIZED VIEW urls_de;

CREATE TABLE urls_de (
    key  TEXT NOT NULL REFERENCES de(key) ON DELETE CASCADE,
    url  TEXT,
    text TEXT
);

CREATE INDEX urls_de_key_idx ON urls_de (key);
