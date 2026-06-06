import dataframely as dy
import polars as pl


class RankingFactorsSchema(dy.Schema):
    id = dy.String(primary_key=True, nullable=False)
    # SMALLINT bounds enforced by `data/processors/search.py` (rank_type<=1100,
    # rank_usage<=900, rank_boost<=99); combined fits comfortably in i16.
    rank_type = dy.Int16(nullable=True)
    rank_combined = dy.Int16(nullable=True)
    rank_usage = dy.Int16(nullable=True)
    rank_custom = dy.Int16(nullable=True)
    rank_boost = dy.Int16(nullable=True)

    @dy.rule()
    def id_non_empty(cls) -> pl.Expr:
        """`id` must be a non-empty entry id."""
        return pl.col("id").str.strip_chars().str.len_chars() > 0
