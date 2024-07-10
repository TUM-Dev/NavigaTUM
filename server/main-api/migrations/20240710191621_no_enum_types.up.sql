-- Add up migration script here
alter table calendar alter column entry_type type text using entry_type::text
