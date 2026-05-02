from pathlib import Path

from external.loaders.events import load_events
from external.schemas.events import EventsSchema

DATA_DIR = Path(__file__).parent.parent
OUTPUT_DIR = DATA_DIR / "output"


def export_events_parquet() -> None:
    """
    Read events.csv, validate, write events.parquet.

    Datetimes are kept as ISO 8601 strings so the Rust parquet reader can parse
    them with chrono::DateTime::parse_from_rfc3339 without depending on Polars
    datetime serialization specifics. EventsSchema enforces the RFC 3339 shape
    and `ends_at >= starts_at` (matching the DB CHECK constraint).
    """
    OUTPUT_DIR.mkdir(exist_ok=True)
    EventsSchema.write_parquet(load_events(), OUTPUT_DIR / "events.parquet")
