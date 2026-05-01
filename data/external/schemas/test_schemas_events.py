import dataframely as dy
import polars as pl
import pytest

from external.loaders.events import load_events
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.events import EventsSchema


def _valid_row() -> dict[str, list]:
    return {
        "image": ["https://example.org/i.jpg"],
        "lat": [48.149678],
        "lon": [11.567909],
        "name": ["Sample"],
        "starts_at": ["2026-06-15T10:00:00+02:00"],
        "ends_at": ["2026-06-15T18:00:00+02:00"],
        "description": ["x"],
        "organising_org_id": [14146],
    }


def test_committed_events_csv_satisfies_schema() -> None:
    """The committed `events.csv` must produce a frame that satisfies `EventsSchema` (drift gate)."""
    assert_satisfies_schema(EventsSchema, load_events())


def test_events_schema_accepts_minimal_valid_row() -> None:
    """A row matching every rule must validate cleanly (positive control)."""
    df = pl.DataFrame(_valid_row(), schema=EventsSchema.to_polars_schema())
    EventsSchema.validate(df)


def test_events_schema_rejects_non_rfc3339_starts_at() -> None:
    """`EventsSchema` must reject `starts_at` values without a timezone offset."""
    row = _valid_row()
    row["starts_at"] = ["2026-06-15 10:00:00"]
    invalid = pl.DataFrame(row, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(invalid)


def test_events_schema_rejects_end_before_start() -> None:
    """`EventsSchema` must reject `ends_at < starts_at` (mirrors the DB CHECK constraint)."""
    row = _valid_row()
    row["ends_at"] = ["2026-06-15T08:00:00+02:00"]
    invalid = pl.DataFrame(row, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(invalid)


def test_events_schema_rejects_non_positive_org_id() -> None:
    """`EventsSchema` must reject rows whose `organising_org_id` is not positive."""
    row = _valid_row()
    row["organising_org_id"] = [0]
    invalid = pl.DataFrame(row, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(invalid)


def test_events_schema_rejects_lat_out_of_range() -> None:
    """`EventsSchema` must reject `lat` values outside [-90, 90]."""
    row = _valid_row()
    row["lat"] = [120.0]
    invalid = pl.DataFrame(row, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(invalid)


def test_events_schema_rejects_missing_column() -> None:
    """`EventsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"name": ["X"]})
    with pytest.raises(dy.exc.SchemaError):
        EventsSchema.validate(incomplete)


@pytest.mark.parametrize("nullable_candidate", ["image", "description"])
def test_events_schema_rejects_null_required_string(nullable_candidate: str) -> None:
    """`image` and `description` are non-null contracts; the schema must enforce that."""
    row = _valid_row()
    row[nullable_candidate] = [None]
    invalid = pl.DataFrame(row, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(invalid)
