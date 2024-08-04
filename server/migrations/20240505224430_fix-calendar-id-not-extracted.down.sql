-- Add down migration script here
-- was never used
create table rooms
(
    key                   text primary key            not null,
    tumonline_org_id      integer                     not null,
    tumonline_calendar_id integer                     not null,
    tumonline_room_id     integer                     not null,
    last_scrape           timestamp without time zone not null
);

-- migrating to
DROP TABLE en;
create table en
(
    key                     text             not null
        primary key
        references de,
    name                    text             not null,
    tumonline_room_nr       integer,
    type                    text             not null,
    type_common_name        text             not null,
    lat                     double precision not null,
    lon                     double precision not null,
    data                    text             not null,
    last_calendar_scrape_at timestamp with time zone
);
comment on column en.last_calendar_scrape_at is 'the last time the calendar was scraped for this room';

DROP TABLE de;
create table de
(
    key                     text             not null primary key,
    name                    text             not null,
    tumonline_room_nr       integer,
    type                    text             not null,
    type_common_name        text             not null,
    lat                     double precision not null,
    lon                     double precision not null,
    data                    text             not null,
    last_calendar_scrape_at timestamp with time zone
);
comment on column de.last_calendar_scrape_at is 'the last time the calendar was scraped for this room';
