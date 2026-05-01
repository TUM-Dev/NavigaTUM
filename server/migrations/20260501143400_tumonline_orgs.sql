CREATE TABLE tumonline_orgs
(
    org_id  INTEGER PRIMARY KEY NOT NULL,
    code    TEXT UNIQUE         NOT NULL,
    name_de TEXT                NOT NULL,
    name_en TEXT                NOT NULL,
    path_de TEXT                NULL,
    path_en TEXT                NULL
);
CREATE INDEX tumonline_orgs_code_idx
    ON tumonline_orgs (code);
