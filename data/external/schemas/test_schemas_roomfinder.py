import dataframely as dy
import polars as pl
import pytest

from external.loaders.roomfinder import load_buildings, load_maps, load_rooms
from external.schemas.roomfinder import BuildingsSchema, MapsSchema, RoomsSchema


def test_committed_buildings_csv_satisfies_schema() -> None:
    """The cached `buildings_roomfinder.csv` must satisfy `BuildingsSchema` (drift gate)."""
    BuildingsSchema.validate(load_buildings())


def test_committed_rooms_csv_satisfies_schema() -> None:
    """The cached `rooms_roomfinder.csv` must satisfy `RoomsSchema` (drift gate)."""
    RoomsSchema.validate(load_rooms())


def test_committed_maps_csv_satisfies_schema() -> None:
    """The cached `maps_roomfinder.csv` must satisfy `MapsSchema` (drift gate)."""
    MapsSchema.validate(load_maps())


def test_buildings_schema_rejects_missing_column() -> None:
    incomplete = pl.DataFrame({"b_id": ["0101"]})
    with pytest.raises(dy.exc.SchemaError):
        BuildingsSchema.validate(incomplete)


def test_rooms_schema_rejects_missing_column() -> None:
    incomplete = pl.DataFrame({"r_id": ["X"]})
    with pytest.raises(dy.exc.SchemaError):
        RoomsSchema.validate(incomplete)


def test_maps_schema_rejects_missing_column() -> None:
    incomplete = pl.DataFrame({"id": ["X"]})
    with pytest.raises(dy.exc.SchemaError):
        MapsSchema.validate(incomplete)
