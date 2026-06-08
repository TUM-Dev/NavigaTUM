import dataframely as dy
import polars as pl

from external.schemas._validators import (
    is_http_url,
    is_iso_date,
    opening_hours_non_empty,
)


class UbTumSchema(dy.Schema):
    """Opening hours scraped from `ub.tum.de`, one row per branch library page."""

    # Stable URL slug (mathematics-informatics, medicine, ...). The branch-to-entry mapping lives in sources/ub_tum_libraries.csv.
    branch_id = dy.String(nullable=False, primary_key=True)
    name = dy.String(nullable=False)
    # Pre-expansion OSM string. May carry lecture:/break: macros and trailing service-variant comments. merge_opening_hours expands the macros.
    opening_hours = dy.String(nullable=False)
    last_update = dy.String(nullable=False)
    source_url = dy.String(nullable=False)

    @dy.rule()
    def opening_hours_non_empty(cls) -> pl.Expr:
        """opening_hours must be a non-empty OSM string after trimming."""
        return opening_hours_non_empty("opening_hours")

    @dy.rule()
    def source_url_is_http(cls) -> pl.Expr:
        """source_url must be an absolute http(s) URL."""
        return is_http_url("source_url")

    @dy.rule()
    def last_update_is_iso_date(cls) -> pl.Expr:
        """last_update must be a YYYY-MM-DD date."""
        return is_iso_date("last_update")
