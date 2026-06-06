import dataframely as dy
import polars as pl

from external.schemas._validators import (
    is_http_url,
    is_iso_date,
    opening_hours_has_no_macros,
    opening_hours_non_empty,
)


class OpeningHoursSchema(dy.Schema):
    """
    Hand-authored opening-hours record attached to a single entry.

    The on-disk form is a plain OSM `opening_hours` string; `id` is the primary
    key, so one schedule per entry. Dates are `YYYY-MM-DD` strings so the server
    reads them from the details JSON without a locale-specific format.
    """

    id = dy.String(nullable=False, primary_key=True)
    opening_hours = dy.String(nullable=False)
    source_url = dy.String(nullable=False)
    last_update = dy.String(nullable=False)
    valid_from = dy.String(nullable=True)
    valid_until = dy.String(nullable=True)
    service = dy.String(nullable=True)

    @dy.rule()
    def opening_hours_non_empty(cls) -> pl.Expr:
        """`opening_hours` must be a non-empty OSM string after trimming."""
        return opening_hours_non_empty("opening_hours")

    @dy.rule()
    def opening_hours_has_no_macros(cls) -> pl.Expr:
        """Reject `lecture:`/`break:` macros; only plain OSM is supported."""
        return opening_hours_has_no_macros("opening_hours")

    @dy.rule()
    def source_url_is_http(cls) -> pl.Expr:
        """`source_url` must be an absolute http(s) URL."""
        return is_http_url("source_url")

    @dy.rule()
    def last_update_is_iso_date(cls) -> pl.Expr:
        """`last_update` must be a `YYYY-MM-DD` date."""
        return is_iso_date("last_update")

    @dy.rule()
    def valid_from_is_iso_date(cls) -> pl.Expr:
        """`valid_from`, when present, must be a `YYYY-MM-DD` date."""
        return pl.col("valid_from").is_null() | is_iso_date("valid_from")

    @dy.rule()
    def valid_until_is_iso_date(cls) -> pl.Expr:
        """`valid_until`, when present, must be a `YYYY-MM-DD` date."""
        return pl.col("valid_until").is_null() | is_iso_date("valid_until")

    @dy.rule()
    def validity_range_ordered(cls) -> pl.Expr:
        """When both bounds are present, `valid_until` must not precede `valid_from`."""
        return (
            pl.col("valid_from").is_null()
            | pl.col("valid_until").is_null()
            | (pl.col("valid_until") >= pl.col("valid_from"))
        )
