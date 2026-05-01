from datetime import datetime
from pathlib import Path

import polars as pl

DATA_DIR = Path(__file__).parent.parent
SOURCES_DIR = DATA_DIR / "sources"
OUTPUT_DIR = DATA_DIR / "output"

EVENTS_CSV = SOURCES_DIR / "events.csv"

EVENT_COLUMNS = [
    "event_image",
    "event_lat",
    "event_lon",
    "event_name",
    "event_datetime_start_at",
    "event_datetime_end_at",
    "event_description",
    "event_organising_org_id",
]


def _validate_iso8601(values: list[str | None], column: str) -> None:
    for i, v in enumerate(values):
        if v is None:
            raise ValueError(f"events.csv row {i}: {column} is required")
        try:
            datetime.fromisoformat(v)
        except ValueError as e:
            raise ValueError(f"events.csv row {i}: {column}={v!r} is not ISO 8601") from e


def export_events_parquet() -> None:
    """Read events.csv, validate, write events.parquet.

    Datetimes are kept as ISO 8601 strings so the Rust parquet reader can parse
    them with chrono::DateTime::parse_from_rfc3339 without depending on Polars
    datetime serialization specifics.
    """
    if not EVENTS_CSV.exists():
        df = pl.DataFrame(
            schema={
                "image": pl.Utf8,
                "lat": pl.Float64,
                "lon": pl.Float64,
                "name": pl.Utf8,
                "starts_at": pl.Utf8,
                "ends_at": pl.Utf8,
                "description": pl.Utf8,
                "organising_org_id": pl.Int32,
            }
        )
    else:
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
        missing = [c for c in EVENT_COLUMNS if c not in raw.columns]
        if missing:
            raise ValueError(f"events.csv missing columns: {missing}")

        _validate_iso8601(raw["event_datetime_start_at"].to_list(), "event_datetime_start_at")
        _validate_iso8601(raw["event_datetime_end_at"].to_list(), "event_datetime_end_at")

        for i, (start, end) in enumerate(
            zip(raw["event_datetime_start_at"], raw["event_datetime_end_at"], strict=True)
        ):
            if datetime.fromisoformat(end) < datetime.fromisoformat(start):
                raise ValueError(f"events.csv row {i}: end {end} is before start {start}")

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

    OUTPUT_DIR.mkdir(exist_ok=True)
    df.write_parquet(OUTPUT_DIR / "events.parquet", use_pyarrow=True, compression_level=22)
