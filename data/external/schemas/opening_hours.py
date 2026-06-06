import dataframely as dy
import polars as pl

_ISO_DATE_REGEX = r"^\d{4}-\d{2}-\d{2}$"


def _is_iso_date(column: str) -> pl.Expr:
    """Zero-padded `YYYY-MM-DD` format *and* a real calendar date (rejects e.g. `2026-13-40`)."""
    col = pl.col(column)
    return col.str.contains(_ISO_DATE_REGEX) & col.str.to_date("%Y-%m-%d", strict=False).is_not_null()


class OpeningHoursSchema(dy.Schema):
    """
    Schema for a hand-authored opening-hours record attached to a single entry.

    The canonical on-disk form is a plain OSM `opening_hours` string (no
    `lecture:`/`break:` macros in this slice). One row per entry: `id` is the
    primary key, which enforces the "one schedule per entry" invariant for now.
    Dates are kept as `YYYY-MM-DD` strings so the server can parse them as plain
    dates from the details JSON without depending on a locale-specific format.
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
        return pl.col("opening_hours").str.strip_chars().str.len_chars() > 0

    @dy.rule()
    def opening_hours_has_no_macros(cls) -> pl.Expr:
        """`lecture:`/`break:` semester macros are out of scope for this slice (plain OSM only)."""
        return ~pl.col("opening_hours").str.contains(r"(?i)\b(lecture|break)\s*:")

    @dy.rule()
    def source_url_is_http(cls) -> pl.Expr:
        """`source_url` must be an absolute http(s) URL the webclient can link to."""
        return pl.col("source_url").str.contains(r"^https?://")

    @dy.rule()
    def last_update_is_iso_date(cls) -> pl.Expr:
        """`last_update` must be a `YYYY-MM-DD` date."""
        return _is_iso_date("last_update")

    @dy.rule()
    def valid_from_is_iso_date(cls) -> pl.Expr:
        """`valid_from`, when present, must be a `YYYY-MM-DD` date."""
        return pl.col("valid_from").is_null() | _is_iso_date("valid_from")

    @dy.rule()
    def valid_until_is_iso_date(cls) -> pl.Expr:
        """`valid_until`, when present, must be a `YYYY-MM-DD` date."""
        return pl.col("valid_until").is_null() | _is_iso_date("valid_until")

    @dy.rule()
    def validity_range_ordered(cls) -> pl.Expr:
        """When both bounds are present, `valid_until` must not precede `valid_from`."""
        return (
            pl.col("valid_from").is_null()
            | pl.col("valid_until").is_null()
            | (pl.col("valid_until") >= pl.col("valid_from"))
        )
