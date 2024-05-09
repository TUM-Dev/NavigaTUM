-- Add down migration script here
alter table en drop constraint en_key_fkey;
alter table en add foreign key (key) references de;

alter table calendar drop constraint calendar_room_code_fkey;
alter table calendar add foreign key (room_code) references en;

alter table aliases drop constraint aliases_key_fkey;
alter table aliases add foreign key (key) references de;
