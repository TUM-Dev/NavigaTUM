DROP MATERIALIZED VIEW usages;

CREATE TABLE usages (
    usage_id     INTEGER NOT NULL,
    name         TEXT    NOT NULL,
    din_277      TEXT,
    din_277_desc TEXT
);

CREATE INDEX usages_id_idx ON usages (usage_id);
