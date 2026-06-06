import logging

import polars as pl

from external.models.iris import IrisRoom
from processors.iris import add_iris_coverage, derive_coverage_building_ids

# A curated slice of the live Iris response (only the two fields the join reads), spanning
# buildings 4113, 3515 and 5606. See `GET https://iris.asta.tum.de/api/` for the full shape.
FIXTURE_ROOMS = [
    IrisRoom(raum_nr_architekt=arch, gebaeude_code=arch.rpartition("@")[2])
    for arch in ("D 11@4113", "D 5@4113", "DG.29@3515", "01.06.011@5606", "01.20@3515", "02.06.020@5606")
]


def test_matched_room_yields_its_building_id():
    """A single Iris room whose arch_name NavigaTUM knows contributes its building id."""
    room = IrisRoom(raum_nr_architekt="D 11@4113", gebaeude_code="4113")

    coverage = derive_coverage_building_ids([room], navigatum_arch_names={"D 11@4113"})

    assert coverage == {"4113"}


def test_room_unknown_to_navigatum_is_ignored():
    """An Iris room whose arch_name NavigaTUM does not know contributes no coverage."""
    room = IrisRoom(raum_nr_architekt="GHOST@9999", gebaeude_code="9999")

    coverage = derive_coverage_building_ids([room], navigatum_arch_names={"D 11@4113"})

    assert coverage == set()


def test_fixture_join_includes_building_with_any_match_excludes_buildings_with_none():
    """
    Against a slice of the live Iris response: a building with >=1 matched room is covered.

    The fixture spans buildings 4113, 3515 and 5606. NavigaTUM here knows one room each from
    4113 and 5606 but none from 3515, so only those two buildings gain coverage.
    """
    coverage = derive_coverage_building_ids(
        FIXTURE_ROOMS,
        navigatum_arch_names={"D 11@4113", "01.06.011@5606"},
    )

    assert coverage == {"4113", "5606"}


def test_building_in_iris_without_alias_match_is_warned_as_coverage_gap(caplog):
    """
    Iris lists a `gebaeude_code` whose rooms NavigaTUM cannot alias-match -> warn, don't cover.

    `gebaeude_code` matches NavigaTUM building ids 1:1, so a building present in Iris but absent
    from the alias-derived set signals that our aliases are missing those rooms.
    """
    rooms = [IrisRoom(raum_nr_architekt="DG.29@3515", gebaeude_code="3515")]

    with caplog.at_level(logging.WARNING):
        coverage = derive_coverage_building_ids(rooms, navigatum_arch_names=set())

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


def test_add_coverage_marks_only_matched_buildings():
    """add_iris_coverage flags the matched building, leaving its rooms and other buildings False."""
    rooms = [IrisRoom(raum_nr_architekt="01.06.011@5606", gebaeude_code="5606")]

    df = add_iris_coverage(_sample_entries(), rooms=rooms)

    coverage = dict(zip(df["id"], df["has_iris_coverage"], strict=True))
    assert coverage == {"5606": True, "5606.EG.011": False, "0001": False}


def test_add_coverage_with_no_rooms_marks_nothing():
    """First build (no scraped roster) marks no coverage and produces a non-null column."""
    df = add_iris_coverage(_sample_entries(), rooms=[])

    assert df["has_iris_coverage"].to_list() == [False, False, False]
    assert df["has_iris_coverage"].null_count() == 0
