ALTER TABLE ranking_factors
    ALTER COLUMN rank_type     TYPE SMALLINT,
    ALTER COLUMN rank_combined TYPE SMALLINT,
    ALTER COLUMN rank_usage    TYPE SMALLINT,
    ALTER COLUMN rank_custom   TYPE SMALLINT,
    ALTER COLUMN rank_boost    TYPE SMALLINT;
