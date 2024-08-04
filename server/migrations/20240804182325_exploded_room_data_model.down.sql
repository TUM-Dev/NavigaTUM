-- Add down migration script here

alter table de drop column coordinate_source;
alter table en drop column coordinate_source;
alter table de drop column rank_type;
alter table en drop column rank_type;
alter table de drop column rank_combined;
alter table en drop column rank_combined;
alter table de drop column rank_usage;
alter table en drop column rank_usage;
alter table de drop column comment;
alter table en drop column comment;

DROP MATERIALIZED VIEW operators_de;
DROP MATERIALIZED VIEW operators_en;
DROP MATERIALIZED VIEW usage;
DROP MATERIALIZED VIEW computed_properties;
DROP MATERIALIZED VIEW urls_de;
DROP MATERIALIZED VIEW urls_en;
DROP MATERIALIZED VIEW sources;
