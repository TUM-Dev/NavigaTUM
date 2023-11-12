-- Add up migration script here
CREATE TABLE aliases
(
    id         SERIAL PRIMARY KEY NOT NULL,
    alias      TEXT                              NOT NULL,
    key        TEXT                              NOT NULL,
    visible_id TEXT                              NOT NULL,
    type       TEXT                              NOT NULL,
    FOREIGN KEY(key) REFERENCES de(key)
);

-- prevent duplicate aliases
CREATE UNIQUE INDEX alias_key ON aliases (alias, key);
