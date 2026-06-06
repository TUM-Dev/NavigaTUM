import json
import logging
from pathlib import Path

import polars as pl

from external.scrapers.iris import IrisRoom
from processors.iris import add_iris_coverage, derive_coverage_building_ids

FIXTURE = Path(__file__).parent / "fixtures" / "iris_sample.json"


def _load_fixture_rooms() -> list[IrisRoom]:
    """Parse the curated slice of the live Iris response into IrisRoom models."""
    payload = json.loads(FIXTURE.read_text(encoding="utf-8"))
    return [IrisRoom.model_validate(raum) for raum in payload["raeume"]]


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
    rooms = _load_fixture_rooms()

    coverage = derive_coverage_building_ids(
        rooms,
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


def test_successful_fetch_marks_covered_buildings_and_persists_set(tmp_path):
    """A reachable Iris build marks matched buildings and writes the set for later fallback."""
    cache = tmp_path / "iris_coverage.csv"
    rooms = [IrisRoom(raum_nr_architekt="01.06.011@5606", gebaeude_code="5606")]

    df = add_iris_coverage(_sample_entries(), fetch=lambda: rooms, cache_path=cache)

    coverage = dict(zip(df["id"], df["has_iris_coverage"], strict=True))
    assert coverage == {"5606": True, "5606.EG.011": False, "0001": False}
    persisted = pl.read_csv(cache, schema_overrides={"building_id": pl.String})
    assert persisted["building_id"].to_list() == ["5606"]


def test_unreachable_iris_falls_back_to_prior_persisted_set(tmp_path):
    """When Iris is unreachable, coverage falls back to the previously-persisted set."""
    cache = tmp_path / "iris_coverage.csv"
    cache.write_text("building_id\n5606\n")

    df = add_iris_coverage(_sample_entries(), fetch=lambda: None, cache_path=cache)

    coverage = dict(zip(df["id"], df["has_iris_coverage"], strict=True))
    assert coverage == {"5606": True, "5606.EG.011": False, "0001": False}


def test_persisted_coverage_round_trip_preserves_leading_zero_ids(tmp_path):
    """Building ids like "0101" must survive the persist -> fallback round-trip as strings."""
    cache = tmp_path / "iris_coverage.csv"
    entries = pl.DataFrame(
        {
            "id": ["0101", "0101.01.101"],
            "type": ["building", "room"],
            "arch_name": ["@0101", "N1101@0101"],
        },
    )
    rooms = [IrisRoom(raum_nr_architekt="N1101@0101", gebaeude_code="0101")]

    # First build persists the set, second build (Iris down) must reload the very same id.
    add_iris_coverage(entries, fetch=lambda: rooms, cache_path=cache)
    df = add_iris_coverage(entries, fetch=lambda: None, cache_path=cache)

    coverage = dict(zip(df["id"], df["has_iris_coverage"], strict=True))
    assert coverage == {"0101": True, "0101.01.101": False}


def test_first_build_with_unreachable_iris_yields_no_coverage(tmp_path):
    """First build (no prior set) with Iris unreachable: empty coverage, no crash."""
    cache = tmp_path / "iris_coverage.csv"

    df = add_iris_coverage(_sample_entries(), fetch=lambda: None, cache_path=cache)

    assert df["has_iris_coverage"].to_list() == [False, False, False]
