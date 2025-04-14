-- Add migration script here
CREATE TABLE aliases
(
    id         SERIAL PRIMARY KEY NOT NULL,
    alias      TEXT   NOT NULL,
    key        TEXT   NOT NULL,
    visible_id TEXT   NOT NULL,
    type       TEXT   NOT NULL,
    FOREIGN KEY(key) REFERENCES de(key)
);

CREATE UNIQUE INDEX aliases_alias_key_uindex ON aliases (alias, key);
