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


