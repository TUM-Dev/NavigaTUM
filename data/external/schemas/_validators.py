"""
Shared dataframely rule expressions for the opening-hours sources.

`OpeningHoursSchema` (hand-authored schedules) and `StudierendenwerkSchema`
(scraped canteen hours) must agree on what a valid OSM `opening_hours` string and
a valid date look like, so the predicates live here rather than being copied into
each schema and drifting apart.
"""

import polars as pl

_ISO_DATE_REGEX = r"^\d{4}-\d{2}-\d{2}$"
# `lecture:`/`break:` macro prefix; `semester_block_expander` reuses this so detection and
# expansion can't drift apart.
MACRO_REGEX = r"(?i)\b(lecture|break)\s*:"


def is_iso_date(column: str) -> pl.Expr:
    """Zero-padded `YYYY-MM-DD` and a real calendar date (rejects e.g. `2026-13-40`)."""
    col = pl.col(column)
    return col.str.contains(_ISO_DATE_REGEX) & col.str.to_date("%Y-%m-%d", strict=False).is_not_null()


def is_http_url(column: str) -> pl.Expr:
    """Match an absolute http(s) URL."""
    return pl.col(column).str.contains(r"^https?://")


def opening_hours_non_empty(column: str) -> pl.Expr:
    """Match a non-empty OSM `opening_hours` string after trimming."""
    return pl.col(column).str.strip_chars().str.len_chars() > 0


def opening_hours_has_no_macros(column: str) -> pl.Expr:
    """Reject `lecture:`/`break:` macros; only plain OSM is supported."""
    return ~pl.col(column).str.contains(MACRO_REGEX)
