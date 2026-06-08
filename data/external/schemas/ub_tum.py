import dataframely as dy
import polars as pl

from external.schemas._validators import (
    is_http_url,
    is_iso_date,
    opening_hours_non_empty,
)


class UbTumSchema(dy.Schema):
    """
    Opening hours scraped from `ub.tum.de` per branch library page.

    `opening_hours` is a canonical OSM string that may carry `lecture:`/`break:`
    macros and per-rule trailing comments (service variants); the macro expansion
    runs in `merge_opening_hours`, so the on-disk form is the pre-expansion shape
    exactly as `OpeningHoursSchema` documents. Keyed by `branch_id`, a stable URL
    slug (`mathematics-informatics`, `medicine`, ...); the branch-to-NavigaTUM-entry
    mapping lives in `sources/ub_tum_libraries.csv`.
    """

    branch_id = dy.String(nullable=False, primary_key=True)
    name = dy.String(nullable=False)
    opening_hours = dy.String(nullable=False)
    last_update = dy.String(nullable=False)
    source_url = dy.String(nullable=False)

    @dy.rule()
    def opening_hours_non_empty(cls) -> pl.Expr:
        """`opening_hours` must be a non-empty OSM string after trimming."""
        return opening_hours_non_empty("opening_hours")

    @dy.rule()
    def source_url_is_http(cls) -> pl.Expr:
        """`source_url` must be an absolute http(s) URL."""
        return is_http_url("source_url")

    @dy.rule()
    def last_update_is_iso_date(cls) -> pl.Expr:
        """`last_update` must be a `YYYY-MM-DD` date."""
        return is_iso_date("last_update")
