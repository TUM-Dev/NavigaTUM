import dataframely as dy
import polars as pl
import pytest

from external.loaders.tumonline import load_usages
from external.schemas.tumonline import UsagesSchema


def test_committed_usages_csv_satisfies_schema() -> None:
    """The cached `usages_tumonline.csv` must satisfy `UsagesSchema` (drift gate)."""
    UsagesSchema.validate(load_usages())


def test_usages_schema_rejects_non_positive_id() -> None:
    invalid = pl.DataFrame(
        {
            "usage_id": [0],
            "din277_id": ["X"],
            "din277_name": ["X"],
            "name": ["X"],
        },
        schema=UsagesSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        UsagesSchema.validate(invalid)


def test_usages_schema_rejects_missing_column() -> None:
    incomplete = pl.DataFrame({"usage_id": [1]})
    with pytest.raises(dy.exc.SchemaError):
        UsagesSchema.validate(incomplete)
