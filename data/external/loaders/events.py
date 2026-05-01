from pathlib import Path

import polars as pl

from external.schemas.events import EventsSchema

SOURCES_PATH = Path(__file__).parent.parent.parent / "sources"
EVENTS_CSV = SOURCES_PATH / "events.csv"


def load_events() -> pl.DataFrame:
    """
    Build the events frame from `data/sources/events.csv`.

    Renames the `event_*`-prefixed CSV columns to the parquet shape and casts
    via `EventsSchema`. Returns an empty schema-conforming frame if the CSV is
    absent so the parquet still gets generated for environments without events.
    """
    if not EVENTS_CSV.exists():
        return EventsSchema.create_empty()

    raw = pl.read_csv(
        EVENTS_CSV,
        schema_overrides={
            "event_image": pl.Utf8,
            "event_lat": pl.Float64,
            "event_lon": pl.Float64,
            "event_name": pl.Utf8,
            "event_datetime_start_at": pl.Utf8,
            "event_datetime_end_at": pl.Utf8,
            "event_description": pl.Utf8,
            "event_organising_org_id": pl.Int32,
        },
    )
    df = raw.rename(
        {
            "event_image": "image",
            "event_lat": "lat",
            "event_lon": "lon",
            "event_name": "name",
            "event_datetime_start_at": "starts_at",
            "event_datetime_end_at": "ends_at",
            "event_description": "description",
            "event_organising_org_id": "organising_org_id",
        }
    ).select(
        "image",
        "lat",
        "lon",
        "name",
        "starts_at",
        "ends_at",
        "description",
        "organising_org_id",
    )
    return EventsSchema.cast(df)
