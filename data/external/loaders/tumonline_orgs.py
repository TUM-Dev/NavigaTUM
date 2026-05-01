import polars as pl

from external.loaders.tumonline import load_orgs
from external.schemas.tumonline_orgs import TumonlineOrgsSchema


def load_tumonline_orgs() -> pl.DataFrame:
    """
    Build the bilingual TUMonline orgs frame from the per-language CSVs.

    Merges `orgs-en_tumonline.csv` and `orgs-de_tumonline.csv` on `org_id`,
    falling back to the English name/path when the German row is missing.
    Dtypes enforced by `TumonlineOrgsSchema`.
    """
    en = load_orgs("en").rename({"name": "name_en", "path": "path_en"})
    de = load_orgs("de").rename({"name": "name_de", "path": "path_de"})
    df = (
        en.join(de.select("org_id", "name_de", "path_de"), on="org_id", how="left")
        .with_columns(
            pl.col("name_de").fill_null(pl.col("name_en")),
            pl.col("path_de").fill_null(pl.col("path_en")),
        )
        .select("org_id", "code", "name_de", "name_en", "path_de", "path_en")
        .unique(subset=["org_id"])
    )
    return TumonlineOrgsSchema.cast(df)
