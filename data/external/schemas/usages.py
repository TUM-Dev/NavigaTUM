import dataframely as dy
import polars as pl


class UsagesSchema(dy.Schema):
    """`usage_id = hashtext(name)` is computed by Postgres at INSERT time."""

    # No primary_key: source data allows the same name to appear with
    # different (din_277, din_277_desc) combinations (~7 cases at current
    # load); the legacy `usages` mat view's UNION tolerates these too.
    name = dy.String(nullable=False)
    din_277 = dy.String(nullable=True)
    din_277_desc = dy.String(nullable=True)

    @dy.rule()
    def name_non_empty(cls) -> pl.Expr:
        """`name` must be a non-empty usage label."""
        return pl.col("name").str.strip_chars().str.len_chars() > 0
