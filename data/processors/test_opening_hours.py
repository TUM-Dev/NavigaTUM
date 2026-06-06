import orjson
import polars as pl
import pytest
from external.schemas.opening_hours import OpeningHoursSchema

from processors.df_utils import unflatten_row
from processors.opening_hours import merge_opening_hours


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
    df = merge_opening_hours(_entries(), schedules=_schedule())

    by_id = dict(zip(df["id"], df["opening_hours_json"], strict=True))
    assert by_id["0001"] is None
    payload = orjson.loads(by_id["5603"])
    assert payload["osm"] == "Mo-Fr 08:00-22:00"
    assert payload["source_url"] == "https://www.ub.tum.de/en/opening-hours"
    assert payload["last_update"] == "2026-06-01"


def test_merge_omits_absent_optional_fields() -> None:
    """Null `valid_from`/`valid_until`/`service` must not appear as null keys in the payload."""
    df = merge_opening_hours(_entries(), schedules=_schedule())

    payload = orjson.loads(dict(zip(df["id"], df["opening_hours_json"], strict=True))["5603"])
    assert "valid_from" not in payload
    assert "valid_until" not in payload
    assert "service" not in payload


def test_merge_keeps_present_optional_fields() -> None:
    """A bounded validity window and service variant must be carried into the payload."""
    schedule = _schedule(valid_from="2026-04-01", valid_until="2026-09-30", service="Lesesaal")
    df = merge_opening_hours(_entries(), schedules=schedule)

    payload = orjson.loads(dict(zip(df["id"], df["opening_hours_json"], strict=True))["5603"])
    assert payload["valid_from"] == "2026-04-01"
    assert payload["valid_until"] == "2026-09-30"
    assert payload["service"] == "Lesesaal"


def test_merge_with_no_schedules_yields_null_column() -> None:
    """With no schedules the column still exists (all null) so downstream export is unconditional."""
    df = merge_opening_hours(_entries(), schedules=_schedule().clear())

    assert df["opening_hours_json"].to_list() == [None, None]


def test_merge_raises_when_schedule_targets_unknown_entry() -> None:
    """A schedule pointing at an id absent from the frame is a typo and must fail the build."""
    orphan = _schedule(id="9999")
    with pytest.raises(ValueError, match="9999"):
        merge_opening_hours(_entries(), schedules=orphan)


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
