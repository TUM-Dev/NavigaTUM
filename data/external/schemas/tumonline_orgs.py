import dataframely as dy
import polars as pl


class TumonlineOrgsSchema(dy.Schema):
    """
    Schema for the merged TUMonline organisation catalogue (`tumonline_orgs.parquet`).

    Built from `orgs-{de,en}_tumonline.csv` joined on `org_id`. The server FK-references
    this table from `events.organising_org_id`.
    """

    org_id = dy.Int32(primary_key=True, nullable=False)
    code = dy.String(nullable=False)
    name_de = dy.String(nullable=False)
    name_en = dy.String(nullable=False)
    path_de = dy.String(nullable=True)
    path_en = dy.String(nullable=True)

    @dy.rule()
    def org_id_positive(cls) -> pl.Expr:
        """`org_id` must be a positive integer."""
        return pl.col("org_id") > 0

    @dy.rule()
    def code_non_empty(cls) -> pl.Expr:
        """`code` must be a non-empty string after trimming."""
        return pl.col("code").str.strip_chars().str.len_chars() > 0
