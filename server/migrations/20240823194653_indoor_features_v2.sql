-- Add up migration script here
-- indoor features v1 was removed as that version hard-depended on geodata being loaded.
-- indoor features v2 does not make this assumption

DROP materialized VIEW IF EXISTS indoor_features CASCADE;
CREATE TABLE indoor_features
(
    group_id bigint PRIMARY KEY NOT NULL UNIQUE,
    features     JSONB NOT NULL,
    convex_hull GEOMETRY NOT NULL,
    import_version bigint NOT NULL default 0
);

CREATE index indoor_features_hull_idx ON indoor_features USING GIST (convex_hull);
