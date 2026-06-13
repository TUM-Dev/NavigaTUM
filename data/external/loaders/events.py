from datetime import datetime
from pathlib import Path

import dataframely as dy
import polars as pl
from processors.events_appears_at import compute_appears_at

from external.schemas.events import EventsSchema

SOURCES_PATH = Path(__file__).parent.parent.parent / "sources"
EVENTS_CSV = SOURCES_PATH / "events.csv"

# Mapping from CSV column names to schema column names.
_CSV_RENAME = {
    "event_image": "image",
    "event_lat": "lat",
    "event_lon": "lon",
    "event_name": "name",
    "event_datetime_start_at": "starts_at",
    "event_datetime_end_at": "ends_at",
    "event_description": "description",
    "event_organising_org_id": "organising_org_id",
    "event_image_author": "image_author",
}


def _appears_at_iso(starts_at: str, ends_at: str) -> str:
    """Return the marker's first-appearance instant as an RFC 3339 string in Europe/Berlin local time."""
    appears_at = compute_appears_at(datetime.fromisoformat(starts_at), datetime.fromisoformat(ends_at))
    return appears_at.isoformat()


def load_events() -> dy.DataFrame[EventsSchema]:
    """
    Build the events frame from `data/sources/events.csv`.

    Renames the `event_*`-prefixed CSV columns to the parquet shape with dtypes
    derived from `EventsSchema` and derives `appears_at` from the event window.
    Validates against the schema so the return type is statically verified by mypy.
    """
    schema = EventsSchema.to_polars_schema()
    read_schema = pl.Schema({csv: schema[parquet] for csv, parquet in _CSV_RENAME.items()})

    df = pl.read_csv(EVENTS_CSV, schema=read_schema).rename(_CSV_RENAME)
    appears_at = [_appears_at_iso(s, e) for s, e in zip(df["starts_at"], df["ends_at"], strict=True)]
    return EventsSchema.validate(df.with_columns(pl.Series("appears_at", appears_at, dtype=pl.String)))
