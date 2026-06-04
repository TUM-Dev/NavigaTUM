import dataframely as dy
import polars as pl


class OperatorsSchema(dy.Schema):
    """Shape shared by `operators_de.parquet` and `operators_en.parquet`."""

    id = dy.Int32(primary_key=True, nullable=False)
    url = dy.String(nullable=True)
    code = dy.String(nullable=True)
    name = dy.String(nullable=True)

    @dy.rule()
    def id_positive(cls) -> pl.Expr:
        return pl.col("id") > 0
