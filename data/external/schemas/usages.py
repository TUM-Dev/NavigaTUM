import dataframely as dy
import polars as pl


class UsagesSchema(dy.Schema):
    """`usage_id = hashtext(name)` is computed by Postgres at INSERT time."""

    name = dy.String(primary_key=True, nullable=False)
    din_277 = dy.String(nullable=True)
    din_277_desc = dy.String(nullable=True)

    @dy.rule()
    def name_non_empty(cls) -> pl.Expr:
        """`name` must be a non-empty usage label."""
        return pl.col("name").str.strip_chars().str.len_chars() > 0
