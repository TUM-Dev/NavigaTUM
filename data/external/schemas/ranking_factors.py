import dataframely as dy
import polars as pl


class RankingFactorsSchema(dy.Schema):
    id = dy.String(primary_key=True, nullable=False)
    rank_type = dy.Int32(nullable=True)
    rank_combined = dy.Int32(nullable=True)
    rank_usage = dy.Int32(nullable=True)
    rank_custom = dy.Int32(nullable=True)
    rank_boost = dy.Int32(nullable=True)

    @dy.rule()
    def id_non_empty(cls) -> pl.Expr:
        return pl.col("id").str.strip_chars().str.len_chars() > 0
