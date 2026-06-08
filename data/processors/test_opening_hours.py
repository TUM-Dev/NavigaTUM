from datetime import date

import opening_hours
import orjson
import polars as pl
import pytest
from external.loaders.opening_hours import load_opening_hours
from external.loaders.semesters import load_semester
from external.schemas.opening_hours import OpeningHoursSchema

from processors.df_utils import unflatten_row
from processors.opening_hours import merge_opening_hours
from processors.semester_block_expander import Semester, contains_macro
from processors.studierendenwerk import mensa_opening_hours
from processors.ub_tum import ub_tum_opening_hours

# A single fixed semester keeps the macro-expansion tests independent of the committed CSV.
_SEMESTER = Semester("2025S", date(2025, 4, 1), date(2025, 4, 22), date(2025, 8, 2), date(2025, 9, 30))


def _entries() -> pl.DataFrame:
    """Build a tiny entry frame: the target library plus an unrelated building."""
    return pl.DataFrame({"id": ["5603", "0001"], "type": ["building", "building"]})


def _schedule(**overrides: object) -> pl.DataFrame:
    """Build a one-row schedule frame matching `OpeningHoursSchema`."""
    row: dict[str, list[object]] = {
        "id": ["5603"],
        "opening_hours": ["Mo-Fr 08:00-22:00"],
        "source_url": ["https://www.ub.tum.de/en/opening-hours"],
        "last_update": ["2026-06-01"],
        "valid_from": [None],
        "valid_until": [None],
        "service": [None],
    }
    for key, value in overrides.items():
        row[key] = [value]
    return pl.DataFrame(row, schema=OpeningHoursSchema.to_polars_schema())


def test_merge_attaches_schedule_to_matching_entry_only() -> None:
    """The matching entry gains an `opening_hours_json` payload; unrelated entries stay null."""
    df = merge_opening_hours(_entries(), schedules=_schedule(), semesters=[])

    by_id = dict(zip(df["id"], df["opening_hours_json"], strict=True))
    assert by_id["0001"] is None
    payload = orjson.loads(by_id["5603"])
    assert payload["osm"] == "Mo-Fr 08:00-22:00"
    assert payload["source_url"] == "https://www.ub.tum.de/en/opening-hours"
    assert payload["last_update"] == "2026-06-01"


def test_merge_omits_absent_optional_fields() -> None:
    """Null `valid_from`/`valid_until`/`service` must not appear as null keys in the payload."""
    df = merge_opening_hours(_entries(), schedules=_schedule(), semesters=[])

    payload = orjson.loads(dict(zip(df["id"], df["opening_hours_json"], strict=True))["5603"])
    assert "valid_from" not in payload
    assert "valid_until" not in payload
    assert "service" not in payload


def test_merge_keeps_present_optional_fields() -> None:
    """A bounded validity window and service variant must be carried into the payload."""
    schedule = _schedule(valid_from="2026-04-01", valid_until="2026-09-30", service="Lesesaal")
    df = merge_opening_hours(_entries(), schedules=schedule, semesters=[])

    payload = orjson.loads(dict(zip(df["id"], df["opening_hours_json"], strict=True))["5603"])
    assert payload["valid_from"] == "2026-04-01"
    assert payload["valid_until"] == "2026-09-30"
    assert payload["service"] == "Lesesaal"


def test_merge_with_no_schedules_yields_null_column() -> None:
    """With no schedules the column still exists, all null."""
    df = merge_opening_hours(_entries(), schedules=_schedule().clear(), semesters=[])

    assert df["opening_hours_json"].to_list() == [None, None]


def test_merge_raises_when_schedule_targets_unknown_entry() -> None:
    """A schedule targeting an id absent from the frame must raise."""
    orphan = _schedule(id="9999")
    with pytest.raises(ValueError, match="9999"):
        merge_opening_hours(_entries(), schedules=orphan, semesters=[])


def test_unflatten_row_lifts_opening_hours_into_entry() -> None:
    """`unflatten_row` must surface the payload as a nested `opening_hours` object on the entry."""
    payload = {"osm": "Mo-Fr 08:00-22:00", "source_url": "https://x.tld", "last_update": "2026-06-01"}
    row = {"id": "5603", "type": "building", "opening_hours_json": orjson.dumps(payload).decode()}

    entry = unflatten_row(row)

    assert entry["opening_hours"] == payload


def test_unflatten_row_omits_opening_hours_when_absent() -> None:
    """An entry without a schedule must not carry an `opening_hours` key."""
    entry = unflatten_row({"id": "0001", "type": "building"})

    assert "opening_hours" not in entry


def test_merge_expands_macros_into_plain_osm_payload() -> None:
    """A `lecture:`/`break:` schedule lands in the payload as expanded, macro-free OSM that parses."""
    schedule = _schedule(opening_hours="lecture: Mo-Fr 08:00-22:00; break: Mo-Fr 09:00-18:00")
    df = merge_opening_hours(_entries(), schedules=schedule, semesters=[_SEMESTER])

    payload = orjson.loads(dict(zip(df["id"], df["opening_hours_json"], strict=True))["5603"])
    assert payload["osm"] == (
        "2025 Apr 22-2025 Aug 02 Mo-Fr 08:00-22:00; "
        "2025 Apr 01-2025 Apr 21 Mo-Fr 09:00-18:00; "
        "2025 Aug 03-2025 Sep 30 Mo-Fr 09:00-18:00"
    )
    assert not contains_macro(payload["osm"])
    assert opening_hours.validate(payload["osm"])


def test_merge_raises_when_macros_cannot_be_expanded() -> None:
    """A macro schedule with no semesters to expand against must not silently ship an empty string."""
    schedule = _schedule(opening_hours="lecture: Mo-Fr 08:00-22:00")
    with pytest.raises(ValueError, match="5603"):
        merge_opening_hours(_entries(), schedules=schedule, semesters=[])


def test_committed_schedules_expand_to_valid_plain_osm() -> None:
    """
    Assert every shipped schedule expands to macro-free OSM that parses.

    Concatenates the same hand-authored / mensa / UB-TUM sources as `compile.py`, so
    this is the single guard that everything reaching `opening_hours_json` is valid
    plain OSM (the compile run itself carries no parser).
    """
    schedules = pl.concat(
        [load_opening_hours(), mensa_opening_hours(), ub_tum_opening_hours()],
        how="vertical",
    ).unique(subset="id", keep="first", maintain_order=True)
    entries = pl.DataFrame({"id": list(schedules["id"]), "type": ["building"] * schedules.height})
    semesters = [Semester.from_row(row) for row in load_semester().iter_rows(named=True)]
    df = merge_opening_hours(entries, schedules=schedules, semesters=semesters)

    payloads = [orjson.loads(value) for value in df["opening_hours_json"] if value is not None]
    assert payloads, "expected at least one committed schedule"
    for payload in payloads:
        assert not contains_macro(payload["osm"]), payload["osm"]
        assert opening_hours.validate(payload["osm"]), payload["osm"]
