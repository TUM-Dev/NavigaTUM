import dataframely as dy
import polars as pl


class ParentsSchema(dy.Schema):
    key = dy.String(nullable=False)
    id = dy.String(nullable=True)
    name = dy.String(nullable=True)

    @dy.rule()
    def key_non_empty(cls) -> pl.Expr:
        return pl.col("key").str.strip_chars().str.len_chars() > 0
