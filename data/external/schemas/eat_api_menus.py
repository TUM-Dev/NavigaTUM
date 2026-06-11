import dataframely as dy
import polars as pl

from external.schemas._validators import is_http_url, is_iso_date

# eat-api dish-type values are short German labels (e.g. `Pasta`, `Suppe`, `Studitopf`); a fixed
# enumeration would silently drop new categories, so the schema only requires non-empty when present.


class EatApiMenuSchema(dy.Schema):
    """
    Per-dish menu rows scraped from the TUM-Dev eat-api weekly canteen feeds.

    eat-api publishes `{canteen}/{year}/{week}.json`, where each day carries an ordered list
    of dishes. We flatten that nesting into one row per dish so the schema can enforce dish-level
    rules; the processor re-nests them per canteen for the response payload. `prices_json` and
    `labels_json` are pre-serialised JSON strings (object resp. array) so the CSV stays a flat
    text file. Keyed by `(canteen_id, date, position)`; `position` is the dish's index within
    the day's list and preserves the upstream serving order.
    """

    canteen_id = dy.String(nullable=False, primary_key=True)
    date = dy.String(nullable=False, primary_key=True)
    position = dy.Integer(nullable=False, primary_key=True, min=0)
    name = dy.String(nullable=False)
    dish_type = dy.String(nullable=True)
    prices_json = dy.String(nullable=False)
    labels_json = dy.String(nullable=False)
    source_url = dy.String(nullable=False)
    last_update = dy.String(nullable=False)

    @dy.rule()
    def name_non_empty(cls) -> pl.Expr:
        """`name` must contain visible characters; an empty title is never a real dish."""
        return pl.col("name").str.strip_chars().str.len_chars() > 0

    @dy.rule()
    def dish_type_non_empty_when_present(cls) -> pl.Expr:
        """`dish_type`, when supplied, must contain visible characters."""
        return pl.col("dish_type").is_null() | (pl.col("dish_type").str.strip_chars().str.len_chars() > 0)

    @dy.rule()
    def date_is_iso(cls) -> pl.Expr:
        """`date` must be a `YYYY-MM-DD` calendar date."""
        return is_iso_date("date")

    @dy.rule()
    def last_update_is_iso_date(cls) -> pl.Expr:
        """`last_update` must be a `YYYY-MM-DD` date."""
        return is_iso_date("last_update")

    @dy.rule()
    def source_url_is_http(cls) -> pl.Expr:
        """`source_url` must be an absolute http(s) URL."""
        return is_http_url("source_url")

    @dy.rule()
    def prices_json_is_object(cls) -> pl.Expr:
        """`prices_json` must be a JSON object string (`{...}`), not a list or scalar."""
        return pl.col("prices_json").str.starts_with("{") & pl.col("prices_json").str.ends_with("}")

    @dy.rule()
    def labels_json_is_array(cls) -> pl.Expr:
        """`labels_json` must be a JSON array string (`[...]`), even when empty."""
        return pl.col("labels_json").str.starts_with("[") & pl.col("labels_json").str.ends_with("]")
