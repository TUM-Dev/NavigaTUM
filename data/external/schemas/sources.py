import dataframely as dy
import polars as pl


class SourcesSchema(dy.Schema):
    key = dy.String(nullable=False)
    url = dy.String(nullable=True)
    name = dy.String(nullable=True)
    patched = dy.Bool(nullable=True)

    @dy.rule()
    def key_non_empty(cls) -> pl.Expr:
        """`key` must be a non-empty entry id."""
        return pl.col("key").str.strip_chars().str.len_chars() > 0
