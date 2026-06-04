from pathlib import Path

import polars as pl
from external.schemas.usages import UsagesSchema

OUTPUT_DIR = Path(__file__).parent.parent.parent / "output"


def export_usages_parquet(df: pl.DataFrame) -> None:
    OUTPUT_DIR.mkdir(exist_ok=True)
    de_side = df.select(
        pl.col("usage_name_de").alias("name"),
        pl.col("usage_din_277").alias("din_277"),
        pl.col("usage_din_277_desc").alias("din_277_desc"),
    )
    en_side = df.select(
        pl.col("usage_name_en").alias("name"),
        pl.col("usage_din_277").alias("din_277"),
        pl.col("usage_din_277_desc").alias("din_277_desc"),
    )
    combined = pl.concat([de_side, en_side]).filter(pl.col("name").is_not_null()).unique()
    UsagesSchema.write_parquet(combined, OUTPUT_DIR / "usages.parquet")
