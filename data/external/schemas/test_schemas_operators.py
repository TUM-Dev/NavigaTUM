import dataframely as dy
import polars as pl
import pytest

from external.schemas.operators import OperatorsSchema


def test_operators_schema_rejects_null_id() -> None:
    """`OperatorsSchema` must reject rows with NULL `id` (PK)."""
    invalid = pl.DataFrame(
        {"id": [None], "url": ["x"], "code": ["x"], "name": ["x"]},
        schema=OperatorsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        OperatorsSchema.validate(invalid)


def test_operators_schema_rejects_non_positive_id() -> None:
    """`OperatorsSchema` must reject non-positive ids."""
    invalid = pl.DataFrame(
        {"id": [0], "url": ["x"], "code": ["x"], "name": ["x"]},
        schema=OperatorsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        OperatorsSchema.validate(invalid)


def test_operators_schema_rejects_duplicate_id() -> None:
    """`id` is the primary key; duplicates must fail validation."""
    duplicates = pl.DataFrame(
        {
            "id": [1, 1],
            "url": ["a", "b"],
            "code": ["a", "b"],
            "name": ["a", "b"],
        },
        schema=OperatorsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        OperatorsSchema.validate(duplicates)


def test_operators_schema_accepts_null_string_columns() -> None:
    """`url`, `code`, `name` must be nullable to match mat-view semantics."""
    valid = pl.DataFrame(
        {"id": [1], "url": [None], "code": [None], "name": [None]},
        schema=OperatorsSchema.to_polars_schema(),
    )
    OperatorsSchema.validate(valid)
