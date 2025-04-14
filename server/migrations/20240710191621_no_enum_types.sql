-- Add migration script here
ALTER TABLE calendar ALTER COLUMN entry_type TYPE TEXT USING entry_type::text
