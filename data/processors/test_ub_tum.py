import logging
from datetime import date

import orjson
import polars as pl
import pytest
from external.schemas.ub_tum import UbTumSchema

from processors.opening_hours import merge_opening_hours
from processors.semester_block_expander import Semester
from processors.ub_tum import ub_tum_opening_hours

_TODAY = date(2026, 6, 8)
# One semester so the macro expander has a date span to attach to in the merge test.
_SEMESTERS = [
    Semester(
        key="2026S",
        start=date(2026, 4, 1),
        lectures_from=date(2026, 4, 13),
        lectures_until=date(2026, 7, 19),
        end=date(2026, 9, 30),
    )
]


def _branches(**overrides: list[object]) -> pl.DataFrame:
    """Build a two-branch UB-TUM scrape frame matching `UbTumSchema`."""
    row: dict[str, list[object]] = {
        "branch_id": ["mathematics-informatics", "medicine"],
        "name": ["Mathematics & Informatics", "Medicine"],
        "opening_hours": [
            "Mo-Fr 08:00-24:00; Sa-Su 10:00-20:00",
            "Mo-Fr 08:00-21:00",
        ],
        "last_update": ["2026-06-08", "2026-06-08"],
        "source_url": [
            "https://www.ub.tum.de/en/branch-library-mathematics-informatics",
            "https://www.ub.tum.de/en/branch-library-medicine",
        ],
    }
    return pl.DataFrame({**row, **overrides}, schema=UbTumSchema.to_polars_schema())


def _mapping(branch_ids: list[str], ids: list[str]) -> pl.DataFrame:
    return pl.DataFrame({"branch_id": branch_ids, "id": ids}, schema={"branch_id": pl.Utf8(), "id": pl.Utf8()})


def test_maps_branch_hours_onto_entry_ids() -> None:
    """A mapped branch yields an `OpeningHoursSchema` record keyed by the NavigaTUM entry id."""
    records = ub_tum_opening_hours(
        branches=_branches(),
        mapping=_mapping(["mathematics-informatics"], ["5603"]),
        today=_TODAY,
    )

    assert records["id"].to_list() == ["5603"]
    record = records.row(0, named=True)
    assert record["opening_hours"] == "Mo-Fr 08:00-24:00; Sa-Su 10:00-20:00"
    assert record["source_url"] == "https://www.ub.tum.de/en/branch-library-mathematics-informatics"
    assert record["valid_from"] is None
    assert record["valid_until"] is None
    assert record["service"] is None


def test_ignores_unmapped_branches() -> None:
    """A branch present in the scrape but absent from the mapping is dropped."""
    records = ub_tum_opening_hours(
        branches=_branches(),
        mapping=_mapping(["mathematics-informatics"], ["5603"]),
        today=_TODAY,
    )
    assert records.height == 1


def test_warns_when_mapping_targets_branch_absent_from_scrape(caplog: pytest.LogCaptureFixture) -> None:
    """A mapping entry with no matching scraped branch is logged and produces no record."""
    with caplog.at_level(logging.WARNING):
        records = ub_tum_opening_hours(
            branches=_branches(),
            mapping=_mapping(["mathematics-informatics", "gone-branch"], ["5603", "9999"]),
            today=_TODAY,
        )
    assert records["id"].to_list() == ["5603"]
    assert "gone-branch" in caplog.text


def test_warns_when_scrape_is_stale(caplog: pytest.LogCaptureFixture) -> None:
    """A scrape older than the staleness window is flagged at build time."""
    with caplog.at_level(logging.WARNING):
        ub_tum_opening_hours(
            branches=_branches(last_update=["2025-01-01", "2025-01-01"]),
            mapping=_mapping(["mathematics-informatics"], ["5603"]),
            today=_TODAY,
        )
    assert "stale" in caplog.text


def test_service_variant_rule_survives_merge_grouped_by_comment() -> None:
    """A trailing-comment service variant lands in `opening_hours_json` so the renderer can group on it."""
    branches = _branches(
        opening_hours=[
            'Mo-Fr 08:00-21:00; Mo-Fr 09:00-20:00 "Pickup of preordered books"',
            "Mo-Fr 08:00-21:00",
        ],
    )
    records = ub_tum_opening_hours(
        branches=branches,
        mapping=_mapping(["mathematics-informatics"], ["5603"]),
        today=_TODAY,
    )
    entries = pl.DataFrame({"id": ["5603"], "type": ["building"]})

    merged = merge_opening_hours(entries, schedules=records, semesters=_SEMESTERS)

    payload = orjson.loads(merged["opening_hours_json"][0])
    assert "Pickup of preordered books" in payload["osm"]


def test_lecture_break_macros_expand_through_merge() -> None:
    """`lecture:`/`break:` macros from the scrape expand against the semester list at merge time."""
    branches = _branches(
        opening_hours=[
            "lecture: Mo-Fr 08:00-24:00; break: Mo-Fr 09:00-20:00",
            "Mo-Fr 08:00-21:00",
        ],
    )
    records = ub_tum_opening_hours(
        branches=branches,
        mapping=_mapping(["mathematics-informatics"], ["5603"]),
        today=_TODAY,
    )
    entries = pl.DataFrame({"id": ["5603"], "type": ["building"]})

    merged = merge_opening_hours(entries, schedules=records, semesters=_SEMESTERS)

    payload = orjson.loads(merged["opening_hours_json"][0])
    # Expanded: lecture covers Apr 13 -> Jul 19; break covers the run-up (Apr 1-12) and tail (Jul 20-Sep 30).
    assert "2026 Apr 13-2026 Jul 19 Mo-Fr 08:00-24:00" in payload["osm"]
    assert "2026 Apr 01-2026 Apr 12 Mo-Fr 09:00-20:00" in payload["osm"]
    assert "2026 Jul 20-2026 Sep 30 Mo-Fr 09:00-20:00" in payload["osm"]
