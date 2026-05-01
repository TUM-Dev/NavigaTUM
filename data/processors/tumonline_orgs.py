from pathlib import Path

import polars as pl

DATA_DIR = Path(__file__).parent.parent
EXTERNAL_RESULTS = DATA_DIR / "external" / "results"
OUTPUT_DIR = DATA_DIR / "output"

ORGS_EN_CSV = EXTERNAL_RESULTS / "orgs-en_tumonline.csv"
ORGS_DE_CSV = EXTERNAL_RESULTS / "orgs-de_tumonline.csv"


def export_tumonline_orgs_parquet() -> None:
    """Read both orgs CSVs, join on org_id, and write tumonline_orgs.parquet."""
    if not ORGS_EN_CSV.exists() or not ORGS_DE_CSV.exists():
        df = pl.DataFrame(
            schema={
                "org_id": pl.Int32,
                "code": pl.Utf8,
                "name_de": pl.Utf8,
                "name_en": pl.Utf8,
                "path_de": pl.Utf8,
                "path_en": pl.Utf8,
            }
        )
    else:
        en = pl.read_csv(ORGS_EN_CSV).rename({"name": "name_en", "path": "path_en"})
        de = pl.read_csv(ORGS_DE_CSV).rename({"name": "name_de", "path": "path_de"})
        df = (
            en.join(de.select("org_id", "name_de", "path_de"), on="org_id", how="left")
            .with_columns(
                pl.col("org_id").cast(pl.Int32),
                pl.col("name_de").fill_null(pl.col("name_en")),
                pl.col("path_de").fill_null(pl.col("path_en")),
            )
            .select("org_id", "code", "name_de", "name_en", "path_de", "path_en")
            .unique(subset=["org_id"])
        )

    OUTPUT_DIR.mkdir(exist_ok=True)
    df.write_parquet(OUTPUT_DIR / "tumonline_orgs.parquet", use_pyarrow=True, compression_level=22)
