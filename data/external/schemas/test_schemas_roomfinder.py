import dataframely as dy
import polars as pl
import pytest

from external.loaders.roomfinder import load_buildings, load_maps, load_rooms
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.roomfinder import BuildingsSchema, MapsSchema, RoomsSchema


def test_committed_buildings_csv_satisfies_schema() -> None:
    """The cached `buildings_roomfinder.csv` must satisfy `BuildingsSchema` (drift gate)."""
    assert_satisfies_schema(BuildingsSchema, load_buildings())


def test_committed_rooms_csv_satisfies_schema() -> None:
    """The cached `rooms_roomfinder.csv` must satisfy `RoomsSchema` (drift gate)."""
    assert_satisfies_schema(RoomsSchema, load_rooms())


def test_committed_maps_csv_satisfies_schema() -> None:
    """The cached `maps_roomfinder.csv` must satisfy `MapsSchema` (drift gate)."""
    assert_satisfies_schema(MapsSchema, load_maps())


def test_buildings_schema_rejects_missing_column() -> None:
    """`BuildingsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"b_id": ["0101"]})
    with pytest.raises(dy.exc.SchemaError):
        BuildingsSchema.validate(incomplete)


def test_rooms_schema_rejects_missing_column() -> None:
    """`RoomsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"r_id": ["X"]})
    with pytest.raises(dy.exc.SchemaError):
        RoomsSchema.validate(incomplete)


def test_maps_schema_rejects_missing_column() -> None:
    """`MapsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"id": ["X"]})
    with pytest.raises(dy.exc.SchemaError):
        MapsSchema.validate(incomplete)
