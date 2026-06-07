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
    """Build a tiny entry frame: a covered building, one of its rooms, and an unrelated building."""
    return pl.DataFrame(
        {
            "id": ["5606", "5606.EG.011", "0001"],
            "type": ["building", "room", "building"],
            "arch_name": ["@5606", "01.06.011@5606", "@0001"],
        }
    )


def test_add_coverage_marks_only_matched_buildings() -> None:
    """add_iris_coverage flags the matched building, leaving its rooms and other buildings False."""
    df = add_iris_coverage(_sample_entries(), rooms=_rooms("01.06.011@5606"))

    coverage = dict(zip(df["id"], df["has_iris_coverage"], strict=True))
    assert coverage == {"5606": True, "5606.EG.011": False, "0001": False}


def test_add_coverage_with_no_rooms_marks_nothing() -> None:
    """First build (no scraped roster) marks no coverage and produces a non-null column."""
    df = add_iris_coverage(_sample_entries(), rooms=_rooms())

    assert df["has_iris_coverage"].to_list() == [False, False, False]
    assert df["has_iris_coverage"].null_count() == 0
