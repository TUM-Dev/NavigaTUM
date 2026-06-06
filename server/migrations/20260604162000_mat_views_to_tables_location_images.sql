DROP MATERIALIZED VIEW location_images;

CREATE TABLE location_images (
    key          TEXT NOT NULL REFERENCES de(key) ON DELETE CASCADE,
    name         TEXT,
    author_url   TEXT,
    author_text  TEXT,
    source_url   TEXT,
    source_text  TEXT,
    license_url  TEXT,
    license_text TEXT
);

CREATE INDEX location_images_key_idx ON location_images (key);
