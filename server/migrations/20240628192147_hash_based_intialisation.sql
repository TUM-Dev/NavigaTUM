-- Add migration script here
ALTER TABLE de ADD hash BIGINT DEFAULT 0; -- the chance of an empty hash is astronomically slim
CREATE INDEX IF NOT EXISTS hash_lut ON de(key, hash);
