-- Add down migration script here
ALTER TABLE en DROP CONSTRAINT en_key_fkey;
ALTER TABLE en ADD FOREIGN KEY (key) REFERENCES de;

ALTER TABLE calendar DROP CONSTRAINT calendar_room_code_fkey;
ALTER TABLE calendar ADD FOREIGN KEY (room_code) REFERENCES en;

ALTER TABLE aliases DROP CONSTRAINT aliases_key_fkey;
ALTER TABLE aliases ADD FOREIGN KEY (key) REFERENCES de;
