-- Add up migration script here
-- indoor features v1 was removed as that version hard-depended on geodata being loaded.
DROP materialized VIEW IF EXISTS indoor_features CASCADE;

-- indoor features v2 does not make this assumption
CREATE TABLE indoor_features
(
    group_id bigint not null,
    features     JSONB NOT NULL,
    convex_hull GEOMETRY NOT NULL,
    import_version bigint NOT NULL default 0
);

alter table indoor_features
    add primary key (group_id, import_version);

CREATE index indoor_features_hull_idx ON indoor_features USING GIST (convex_hull);
