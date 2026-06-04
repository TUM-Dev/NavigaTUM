import dataframely as dy
import polars as pl


class UrlsSchema(dy.Schema):
    """Shape shared by `urls_de.parquet` and `urls_en.parquet`."""

    key = dy.String(nullable=False)
    url = dy.String(nullable=True)
    text = dy.String(nullable=True)

    @dy.rule()
    def key_non_empty(cls) -> pl.Expr:
        return pl.col("key").str.strip_chars().str.len_chars() > 0
