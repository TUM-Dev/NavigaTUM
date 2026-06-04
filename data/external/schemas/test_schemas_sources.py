import dataframely as dy
import polars as pl
import pytest

from external.schemas.sources import SourcesSchema


def test_sources_schema_rejects_empty_key() -> None:
    """`SourcesSchema` must reject rows with an empty `key`."""
    invalid = pl.DataFrame(
        {"key": [""], "url": ["x"], "name": ["x"], "patched": [True]},
        schema=SourcesSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        SourcesSchema.validate(invalid)


def test_sources_schema_accepts_null_url_name_patched() -> None:
    """url/name/patched must be nullable to match mat-view semantics."""
    valid = pl.DataFrame(
        {"key": ["x"], "url": [None], "name": [None], "patched": [None]},
        schema=SourcesSchema.to_polars_schema(),
    )
    SourcesSchema.validate(valid)


def test_sources_schema_accepts_duplicate_key() -> None:
    """No primary key: an entry can have multiple source rows."""
    valid = pl.DataFrame(
        {
            "key": ["x", "x"],
            "url": ["a", "b"],
            "name": ["A", "B"],
            "patched": [True, True],
        },
        schema=SourcesSchema.to_polars_schema(),
    )
    SourcesSchema.validate(valid)
