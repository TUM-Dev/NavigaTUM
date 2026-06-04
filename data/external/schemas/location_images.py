import dataframely as dy
import polars as pl


class LocationImagesSchema(dy.Schema):
    key = dy.String(nullable=False)
    name = dy.String(nullable=True)
    author_url = dy.String(nullable=True)
    author_text = dy.String(nullable=True)
    source_url = dy.String(nullable=True)
    source_text = dy.String(nullable=True)
    license_url = dy.String(nullable=True)
    license_text = dy.String(nullable=True)

    @dy.rule()
    def key_non_empty(cls) -> pl.Expr:
        return pl.col("key").str.strip_chars().str.len_chars() > 0
