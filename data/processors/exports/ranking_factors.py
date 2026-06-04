from pathlib import Path

import polars as pl
from external.schemas.ranking_factors import RankingFactorsSchema

OUTPUT_DIR = Path(__file__).parent.parent.parent / "output"


def export_ranking_factors_parquet(df: pl.DataFrame) -> None:
    """Write `ranking_factors.parquet` from the flat location dataframe."""
    OUTPUT_DIR.mkdir(exist_ok=True)
    extracted = df.select(
        pl.col("id"),
        pl.col("ranking_rank_type").cast(pl.Int16).alias("rank_type"),
        pl.col("ranking_rank_combined").cast(pl.Int16).alias("rank_combined"),
        pl.col("ranking_rank_usage").cast(pl.Int16).alias("rank_usage"),
        pl.col("ranking_rank_custom").cast(pl.Int16).alias("rank_custom"),
        pl.col("ranking_rank_boost").cast(pl.Int16).alias("rank_boost"),
    ).unique()
    RankingFactorsSchema.write_parquet(extracted, OUTPUT_DIR / "ranking_factors.parquet")
