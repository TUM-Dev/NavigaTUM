import dataframely as dy
import polars as pl
import pytest

from external.schemas.parents import ParentsSchema


def test_parents_schema_rejects_empty_key() -> None:
    """`ParentsSchema` must reject rows with an empty `key`."""
    invalid = pl.DataFrame(
        {"key": [""], "id": ["root"], "name": ["Standorte"]},
        schema=ParentsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        ParentsSchema.validate(invalid)


def test_parents_schema_accepts_multiple_rows_per_key() -> None:
    """Multiple ancestors per entry is the whole point."""
    valid = pl.DataFrame(
        {
            "key": ["0101", "0101"],
            "id": ["root", "stammgelaende"],
            "name": ["Standorte", "Stammgelände"],
        },
        schema=ParentsSchema.to_polars_schema(),
    )
    ParentsSchema.validate(valid)


def test_parents_schema_accepts_null_id_name() -> None:
    """`id` / `name` are nullable to match mat-view permissiveness."""
    valid = pl.DataFrame(
        {"key": ["x"], "id": [None], "name": [None]},
        schema=ParentsSchema.to_polars_schema(),
    )
    ParentsSchema.validate(valid)
