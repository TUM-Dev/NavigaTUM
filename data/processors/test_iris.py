import logging

import polars as pl
import pytest
from external.schemas.iris import IrisRoomsSchema

from processors.iris import add_iris_coverage, derive_coverage_building_ids


def _rooms(*raum_nr_architekt: str) -> pl.DataFrame:
    """Build an Iris roster frame; gebaeude_code is the `@`-suffix, as Iris reports it."""
    rows = [{"raum_nr_architekt": arch, "gebaeude_code": arch.rpartition("@")[2]} for arch in raum_nr_architekt]
    return pl.DataFrame(rows, schema=IrisRoomsSchema.to_polars_schema())


def test_matched_room_yields_its_building_id() -> None:
    """A single Iris room whose arch_name NavigaTUM knows contributes its building id."""
    coverage = derive_coverage_building_ids(_rooms("D 11@4113"), navigatum_arch_names={"D 11@4113"})

    assert coverage == {"4113"}


def test_room_unknown_to_navigatum_is_ignored() -> None:
    """An Iris room whose arch_name NavigaTUM does not know contributes no coverage."""
    coverage = derive_coverage_building_ids(_rooms("GHOST@9999"), navigatum_arch_names={"D 11@4113"})

    assert coverage == set()


def test_join_includes_building_with_any_match_excludes_buildings_with_none() -> None:
    """
    A building with >=1 matched room is covered; a building with none is not.

    The roster spans buildings 4113, 3515 and 5606. NavigaTUM here knows one room each from
    4113 and 5606 but none from 3515, so only those two buildings gain coverage.
    """
    rooms = _rooms("D 11@4113", "D 5@4113", "DG.29@3515", "01.06.011@5606", "01.20@3515", "02.06.020@5606")

    coverage = derive_coverage_building_ids(rooms, navigatum_arch_names={"D 11@4113", "01.06.011@5606"})

    assert coverage == {"4113", "5606"}


def test_building_in_iris_without_alias_match_is_warned_as_coverage_gap(caplog: pytest.LogCaptureFixture) -> None:
    """
    Iris lists a `gebaeude_code` whose rooms NavigaTUM cannot alias-match -> warn, don't cover.

    `gebaeude_code` matches NavigaTUM building ids 1:1, so a building present in Iris but absent
    from the alias-derived set signals that our aliases are missing those rooms.
    """
    with caplog.at_level(logging.WARNING):
        coverage = derive_coverage_building_ids(_rooms("DG.29@3515"), navigatum_arch_names=set())

    assert coverage == set()
    assert "3515" in caplog.text


def _sample_entries() -> pl.DataFrame:
    """
    Build a tiny entry frame mirroring the MI shape.

    The `mi` joined_building has child building `5606` (holding a room) and an empty sibling `5607`.
    `0001` is an unrelated building.
    `children_flat` is null for leaves, matching the left-join in `structure.add_children_properties`.
    """
    return pl.DataFrame(
        {
            "id": ["mi", "5606", "5607", "5606.EG.011", "0001"],
            "type": ["joined_building", "building", "building", "room", "building"],
            "arch_name": [None, "@5606", "@5607", "01.06.011@5606", "@0001"],
            "children_flat": [["5606", "5607", "5606.EG.011"], None, None, None, None],
        }
    )


def test_add_coverage_lists_matched_buildings_and_propagates_to_joined_building() -> None:
    """
    The matched building lists itself and its joined_building parent inherits it.

    Regression for the MI bug, where `mi` stayed empty although child `5606` has a learning room.
    Empty siblings, rooms, and unrelated buildings stay empty.
    """
    df = add_iris_coverage(_sample_entries(), rooms=_rooms("01.06.011@5606"))

    coverage = dict(zip(df["id"].to_list(), df["iris_coverage_building_ids"].to_list(), strict=True))
    assert coverage == {
        "mi": ["5606"],
        "5606": ["5606"],
        "5607": [],
        "5606.EG.011": [],
        "0001": [],
    }


def test_add_coverage_with_no_rooms_marks_nothing() -> None:
    """First build (no scraped roster) lists no coverage and produces a non-null column."""
    df = add_iris_coverage(_sample_entries(), rooms=_rooms())

    assert df["iris_coverage_building_ids"].to_list() == [[], [], [], [], []]
    assert df["iris_coverage_building_ids"].null_count() == 0
