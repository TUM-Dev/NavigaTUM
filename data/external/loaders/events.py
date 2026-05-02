from pathlib import Path

import polars as pl

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
}


def load_events() -> pl.DataFrame:
    """
    Build the events frame from `data/sources/events.csv`.

    Renames the `event_*`-prefixed CSV columns to the parquet shape with dtypes
    derived from `EventsSchema`.
    """
    schema = EventsSchema.to_polars_schema()
    read_schema = pl.Schema({csv: schema[parquet] for csv, parquet in _CSV_RENAME.items()})

    df = pl.read_csv(EVENTS_CSV, schema=read_schema)
    return df.rename(_CSV_RENAME)
