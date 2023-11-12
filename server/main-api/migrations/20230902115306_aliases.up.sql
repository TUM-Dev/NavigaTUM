-- Add up migration script here
CREATE TABLE aliases
(
    id         SERIAL PRIMARY KEY NOT NULL,
    alias      TEXT UNIQUE                       NOT NULL,
    key        TEXT                              NOT NULL,
    visible_id TEXT                              NOT NULL,
    type       TEXT                              NOT NULL,
    FOREIGN KEY(key) REFERENCES de(key)
);
