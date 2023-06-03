-- This file should undo anything in `down.sql`
ALTER TABLE calendar
DROP COLUMN tumonline_id;
