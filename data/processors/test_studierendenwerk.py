import logging
from datetime import date

import orjson
import polars as pl
import pytest
from external.schemas.studierendenwerk import StudierendenwerkSchema

from processors.opening_hours import merge_opening_hours
from processors.studierendenwerk import mensa_opening_hours, stamp_canteen_ids

_TODAY = date(2026, 6, 7)


def _canteens(**overrides: list[object]) -> pl.DataFrame:
    """Build a two-canteen feed frame matching `StudierendenwerkSchema`."""
    row: dict[str, list[object]] = {
        "canteen_id": ["mensa-garching", "mensa-arcisstr"],
        "name": ["Mensa Garching", "Mensa Arcisstraße"],
        "opening_hours": ["Mo-Fr 10:45-14:15", "Mo-Fr 11:00-14:00"],
        "last_update": ["2026-06-05", "2026-06-05"],
        "source_url": [
            "https://tum-dev.github.io/eat-api/#!/de/mensa-garching",
            "https://tum-dev.github.io/eat-api/#!/de/mensa-arcisstr",
        ],
    }
    return pl.DataFrame({**row, **overrides}, schema=StudierendenwerkSchema.to_polars_schema())


def _mapping(canteen_ids: list[str], ids: list[str]) -> pl.DataFrame:
    return pl.DataFrame({"canteen_id": canteen_ids, "id": ids}, schema={"canteen_id": pl.Utf8(), "id": pl.Utf8()})


def test_maps_canteen_hours_onto_entry_ids() -> None:
    """A mapped canteen yields an `OpeningHoursSchema` record keyed by the NavigaTUM entry id."""
    records = mensa_opening_hours(
        canteens=_canteens(),
        mapping=_mapping(["mensa-garching"], ["5304"]),
        today=_TODAY,
    )

    assert records["id"].to_list() == ["5304"]
    record = records.row(0, named=True)
    assert record["opening_hours"] == "Mo-Fr 10:45-14:15"
    assert record["source_url"] == "https://tum-dev.github.io/eat-api/#!/de/mensa-garching"
    assert record["valid_from"] is None
    assert record["valid_until"] is None
    assert record["service"] is None


def test_ignores_unmapped_canteens() -> None:
    """A canteen present in the feed but absent from the mapping is dropped."""
    records = mensa_opening_hours(
        canteens=_canteens(),
        mapping=_mapping(["mensa-garching"], ["5304"]),
        today=_TODAY,
    )
    assert records.height == 1


def test_warns_when_mapping_targets_canteen_absent_from_feed(caplog: pytest.LogCaptureFixture) -> None:
    """A mapping entry with no matching feed canteen is logged and produces no record."""
    with caplog.at_level(logging.WARNING):
        records = mensa_opening_hours(
            canteens=_canteens(),
            mapping=_mapping(["mensa-garching", "mensa-gone"], ["5304", "9999"]),
            today=_TODAY,
        )
    assert records["id"].to_list() == ["5304"]
    assert "mensa-gone" in caplog.text


def test_warns_when_feed_is_stale(caplog: pytest.LogCaptureFixture) -> None:
    """A feed snapshot older than the staleness window is flagged at build time."""
    with caplog.at_level(logging.WARNING):
        mensa_opening_hours(
            canteens=_canteens(last_update=["2025-01-01", "2025-01-01"]),
            mapping=_mapping(["mensa-garching"], ["5304"]),
            today=_TODAY,
        )
    assert "stale" in caplog.text


def test_records_merge_onto_entries() -> None:
    """The produced records attach through the shared opening-hours merge."""
    records = mensa_opening_hours(
        canteens=_canteens(),
        mapping=_mapping(["mensa-garching"], ["5304"]),
        today=_TODAY,
    )
    entries = pl.DataFrame({"id": ["5304", "0001"], "type": ["building", "building"]})

    merged = merge_opening_hours(entries, schedules=records)

    by_id = dict(zip(merged["id"], merged["opening_hours_json"], strict=True))
    assert by_id["0001"] is None
    payload = orjson.loads(by_id["5304"])
    assert payload["osm"] == "Mo-Fr 10:45-14:15"
    assert "valid_from" not in payload


def test_stamps_canteen_slug_onto_mapped_entries() -> None:
    """Mapped entries get the eat-api slug; everything else stays null."""
    entries = pl.DataFrame({"id": ["5304", "0206", "0001"]})

    stamped = stamp_canteen_ids(
        entries,
        mapping=_mapping(["mensa-garching", "mensa-arcisstr"], ["5304", "0206"]),
    )

    by_id = dict(zip(stamped["id"], stamped["mensa_canteen_id"], strict=True))
    assert by_id == {"5304": "mensa-garching", "0206": "mensa-arcisstr", "0001": None}


def test_stamp_rejects_mapping_to_unknown_entry() -> None:
    """A mapping targeting an id absent from the entries fails the build (mapping drift)."""
    entries = pl.DataFrame({"id": ["5304"]})

    with pytest.raises(ValueError, match="9999"):
        stamp_canteen_ids(entries, mapping=_mapping(["mensa-garching", "mensa-gone"], ["5304", "9999"]))
