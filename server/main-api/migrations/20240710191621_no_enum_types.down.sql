-- Add down migration script here
DELETE FROM calendar WHERE 1=1 -- to make migrating simpler and because it is possible
alter table calendar alter column entry_type type entry_type;
