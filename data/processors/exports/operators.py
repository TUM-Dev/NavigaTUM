from pathlib import Path

import polars as pl

from external.schemas.operators import OperatorsSchema

OUTPUT_DIR = Path(__file__).parent.parent.parent / "output"


def export_operators_de_parquet(df: pl.DataFrame) -> None:
    _write_operators_parquet(df, name_column="props_operator_name_de", filename="operators_de.parquet")


def export_operators_en_parquet(df: pl.DataFrame) -> None:
    _write_operators_parquet(df, name_column="props_operator_name_en", filename="operators_en.parquet")


def _write_operators_parquet(df: pl.DataFrame, *, name_column: str, filename: str) -> None:
    OUTPUT_DIR.mkdir(exist_ok=True)
    extracted = (
        df.select(
            pl.col("props_operator_id").cast(pl.Int32).alias("id"),
            pl.col("props_operator_url").alias("url"),
            pl.col("props_operator_code").alias("code"),
            pl.col(name_column).alias("name"),
        )
        .filter(pl.col("id").is_not_null())
        .unique()
    )
    OperatorsSchema.write_parquet(extracted, OUTPUT_DIR / filename)
