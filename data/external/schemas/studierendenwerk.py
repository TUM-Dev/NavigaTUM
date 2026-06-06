import dataframely as dy
import polars as pl

from external.schemas._validators import (
    is_http_url,
    is_iso_date,
    opening_hours_has_no_macros,
    opening_hours_non_empty,
)


class StudierendenwerkSchema(dy.Schema):
    """
    Canteen opening hours scraped from the TUM-Dev eat-api `canteens.json` feed.

    Studierendenwerk München publishes no structured feed, so eat-api's MIT-licensed
    JSON (which scrapes the HTML for us) is the source. `opening_hours` is already a
    plain OSM string built from the feed's per-weekday start/end; the OSM-string parse
    is gated by the test suite, not at runtime, exactly as for `OpeningHoursSchema`.
    Keyed by the eat-api `canteen_id`; the canteen-to-NavigaTUM-entry mapping lives in
    `sources/mensa_canteens.csv`.
    """

    canteen_id = dy.String(nullable=False, primary_key=True)
    name = dy.String(nullable=False)
    opening_hours = dy.String(nullable=False)
    last_update = dy.String(nullable=False)
    source_url = dy.String(nullable=False)

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
