import typing

import dataframely as dy
import polars as pl
import pytest

from external.loaders.tumonline import load_buildings, load_orgs, load_rooms, load_usages
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.tumonline import BuildingsSchema, OrgsSchema, RoomsSchema, UsagesSchema


def test_committed_usages_csv_satisfies_schema() -> None:
    """The cached `usages_tumonline.csv` must satisfy `UsagesSchema` (drift gate)."""
    assert_satisfies_schema(UsagesSchema, load_usages())


def test_usages_schema_rejects_non_positive_id() -> None:
    """`UsagesSchema` must reject rows with a non-positive `usage_id`."""
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
    """`UsagesSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"usage_id": [1]})
    with pytest.raises(dy.exc.SchemaError):
        UsagesSchema.validate(incomplete)


@pytest.mark.parametrize("lang", ["de", "en"])
def test_committed_orgs_csv_satisfies_schema(lang: typing.Literal["de", "en"]) -> None:
    """The cached `orgs-{lang}_tumonline.csv` must satisfy `OrgsSchema` (drift gate)."""
    assert_satisfies_schema(OrgsSchema, load_orgs(lang))


def test_orgs_schema_rejects_non_positive_id() -> None:
    """`OrgsSchema` must reject rows with a non-positive `org_id`."""
    invalid = pl.DataFrame(
        {"org_id": [0], "code": ["X"], "name": ["X"], "path": ["X"]},
        schema=OrgsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        OrgsSchema.validate(invalid)


def test_orgs_schema_rejects_missing_column() -> None:
    """`OrgsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"org_id": [1]})
    with pytest.raises(dy.exc.SchemaError):
        OrgsSchema.validate(incomplete)


def test_committed_buildings_csv_satisfies_schema() -> None:
    """The cached `buildings_tumonline.csv` must satisfy `BuildingsSchema` (drift gate)."""
    assert_satisfies_schema(BuildingsSchema, load_buildings())


def test_buildings_schema_rejects_non_four_digit_key() -> None:
    """`BuildingsSchema` must reject rows whose `building_key` isn't four digits."""
    invalid = pl.DataFrame(
        {
            "building_key": ["12"],
            "address_place": ["X"],
            "address_street": ["X"],
            "address_zip_code": [80333],
            "area_id": [1],
            "name": ["X"],
            "tumonline_id": [1],
            "filter_id": [None],
        },
        schema=BuildingsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        BuildingsSchema.validate(invalid)


def test_buildings_schema_rejects_missing_column() -> None:
    """`BuildingsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"building_key": ["0101"]})
    with pytest.raises(dy.exc.SchemaError):
        BuildingsSchema.validate(incomplete)


def test_committed_rooms_csv_satisfies_schema() -> None:
    """The cached `rooms_tumonline.csv` must satisfy `RoomsSchema` (drift gate)."""
    assert_satisfies_schema(RoomsSchema, load_rooms())


def test_rooms_schema_rejects_non_positive_tumonline_id() -> None:
    """`RoomsSchema` must reject rows with a non-positive `tumonline_id`."""
    invalid = pl.DataFrame(
        {
            "room_key": ["0101.01.101"],
            "address_place": ["X"],
            "address_street": ["X"],
            "address_zip_code": [80333],
            "seats_sitting": [None],
            "seats_wheelchair": [None],
            "seats_standing": [None],
            "floor_type": ["X"],
            "floor_level": ["X"],
            "tumonline_id": [0],
            "area_id": [1],
            "building_id": [1],
            "main_operator_id": [1],
            "usage_id": [1],
            "alt_name": [None],
            "arch_name": [None],
            "calendar_resource_nr": [None],
            "patched": [False],
        },
        schema=RoomsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        RoomsSchema.validate(invalid)


def test_rooms_schema_rejects_missing_column() -> None:
    """`RoomsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"room_key": ["0101.01.101"]})
    with pytest.raises(dy.exc.SchemaError):
        RoomsSchema.validate(incomplete)
