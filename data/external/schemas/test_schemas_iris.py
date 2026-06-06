import dataframely as dy
import polars as pl
import pytest

from external.loaders.iris import load_iris_rooms
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.iris import IrisRoomsSchema


def test_committed_iris_csv_satisfies_schema() -> None:
    """The cached `iris.csv` must satisfy `IrisRoomsSchema` (drift gate)."""
    assert_satisfies_schema(IrisRoomsSchema, load_iris_rooms())


def test_iris_schema_rejects_duplicate_room() -> None:
    """`IrisRoomsSchema` must reject a roster with a duplicated `raum_nr_architekt`."""
    duplicated = pl.DataFrame(
        {"raum_nr_architekt": ["N1@0101", "N1@0101"], "gebaeude_code": ["0101", "0101"]},
        schema=IrisRoomsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        IrisRoomsSchema.validate(duplicated)


def test_iris_schema_rejects_missing_column() -> None:
    """`IrisRoomsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"raum_nr_architekt": ["N1@0101"]})
    with pytest.raises(dy.exc.SchemaError):
        IrisRoomsSchema.validate(incomplete)
