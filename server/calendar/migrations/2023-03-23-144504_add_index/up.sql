-- Your SQL goes here
CREATE INDEX IF NOT EXISTS calendar_lut ON calendar(key, dtstart, dtend)