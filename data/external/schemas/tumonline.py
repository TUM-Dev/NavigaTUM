import dataframely as dy
import polars as pl


class UsagesSchema(dy.Schema):
    """Schema for the TUMonline usage type catalogue (`usages_tumonline.csv`)."""

    usage_id = dy.Int64(primary_key=True, nullable=False)
    din277_id = dy.String(nullable=False)
    din277_name = dy.String(nullable=False)
    name = dy.String(nullable=False)

    @dy.rule()
    def usage_id_positive(cls) -> pl.Expr:
        return pl.col("usage_id") > 0


class OrgsSchema(dy.Schema):
    """Schema for the TUMonline organisation catalogue (`orgs-{lang}_tumonline.csv`)."""

    org_id = dy.Int64(primary_key=True, nullable=False)
    code = dy.String(nullable=False)
    name = dy.String(nullable=False)
    path = dy.String(nullable=False)

    @dy.rule()
    def org_id_positive(cls) -> pl.Expr:
        return pl.col("org_id") > 0
