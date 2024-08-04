-- Add up migration script here
alter table de add hash BIGINT default 0; -- the chance of an empty hash is astronomically slim
CREATE INDEX IF NOT EXISTS hash_lut ON de(key, hash);
