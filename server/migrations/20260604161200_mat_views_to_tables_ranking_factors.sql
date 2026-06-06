DROP MATERIALIZED VIEW ranking_factors;

CREATE TABLE ranking_factors (
    id            TEXT    PRIMARY KEY REFERENCES de(key) ON DELETE CASCADE,
    rank_type     INTEGER,
    rank_combined INTEGER,
    rank_usage    INTEGER,
    rank_custom   INTEGER,
    rank_boost    INTEGER
);
