import dataframely as dy
import polars as pl
import pytest

from external.schemas.usages import UsagesSchema


def test_usages_schema_rejects_empty_name() -> None:
    """`UsagesSchema` must reject rows with an empty `name`."""
    invalid = pl.DataFrame(
        {"name": [""], "din_277": ["X"], "din_277_desc": ["X"]},
        schema=UsagesSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        UsagesSchema.validate(invalid)


def test_usages_schema_rejects_duplicate_name() -> None:
    """Same name with different DIN values is a drift signal - must fail."""
    invalid = pl.DataFrame(
        {
            "name": ["NF1.2", "NF1.2"],
            "din_277": ["A", "B"],
            "din_277_desc": ["a", "b"],
        },
        schema=UsagesSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        UsagesSchema.validate(invalid)


def test_usages_schema_accepts_null_din() -> None:
    """`din_277` / `din_277_desc` must be nullable."""
    valid = pl.DataFrame(
        {"name": ["NF1.2"], "din_277": [None], "din_277_desc": [None]},
        schema=UsagesSchema.to_polars_schema(),
    )
    UsagesSchema.validate(valid)
