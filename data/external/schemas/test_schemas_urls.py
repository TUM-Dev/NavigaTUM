import dataframely as dy
import polars as pl
import pytest

from external.schemas.urls import UrlsSchema


def test_urls_schema_rejects_empty_key() -> None:
    """`UrlsSchema` must reject rows with an empty `key`."""
    invalid = pl.DataFrame(
        {"key": [""], "url": ["x"], "text": ["x"]},
        schema=UrlsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        UrlsSchema.validate(invalid)


def test_urls_schema_accepts_null_url_text() -> None:
    """url/text must be nullable (some link entries may lack a language variant)."""
    valid = pl.DataFrame(
        {"key": ["x"], "url": [None], "text": [None]},
        schema=UrlsSchema.to_polars_schema(),
    )
    UrlsSchema.validate(valid)


def test_urls_schema_accepts_duplicate_key() -> None:
    """An entry can have multiple link rows."""
    valid = pl.DataFrame(
        {
            "key": ["x", "x"],
            "url": ["https://a", "https://b"],
            "text": ["A", "B"],
        },
        schema=UrlsSchema.to_polars_schema(),
    )
    UrlsSchema.validate(valid)
