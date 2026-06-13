import dataframely as dy
import polars as pl
import pytest

from external.loaders.events import load_events
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.events import EventsSchema


def _valid_row() -> dict[str, list[object]]:
    return {
        "image": ["/cdn/thumb/event_abc123def4567890_0.webp"],
        "lat": [48.149678],
        "lon": [11.567909],
        "name": ["Sample"],
        "starts_at": ["2026-06-15T10:00:00+02:00"],
        "ends_at": ["2026-06-15T18:00:00+02:00"],
        "appears_at": ["2026-06-13T10:00:00+02:00"],
        "description": ["x"],
        "organising_org_id": [14146],
        "image_author": ["Studi"],
    }


def test_events_schema_rejects_blank_image_author() -> None:
    """CC-BY enforcement: a blank or whitespace-only `image_author` must fail validation."""
    invalid = _valid_row()
    invalid["image_author"] = ["   "]
    df = pl.DataFrame(invalid, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(df)


def test_committed_events_csv_satisfies_schema() -> None:
    """The committed `events.csv` must produce a frame that satisfies `EventsSchema` (drift gate)."""
    assert_satisfies_schema(EventsSchema, load_events())


def test_events_schema_accepts_minimal_valid_row() -> None:
    """A row matching every rule must validate cleanly (positive control)."""
    df = pl.DataFrame(_valid_row(), schema=EventsSchema.to_polars_schema())
    EventsSchema.validate(df)


def test_events_schema_accepts_event_thumb_name() -> None:
    """The `event_<hash>` base name (with its underscore) is a valid thumb path."""
    valid = _valid_row()
    valid["image"] = ["/cdn/thumb/event_9d02ddd940c43f87_0.webp"]
    df = pl.DataFrame(valid, schema=EventsSchema.to_polars_schema())
    EventsSchema.validate(df)


@pytest.mark.parametrize(
    "image",
    [
        "https://example.org/i.webp",  # external host
        "event_9d02ddd940c43f87_0.webp",  # bare filename, missing the /cdn/thumb/ delivery prefix
        "/cdn/lg/event_9d02ddd940c43f87_0.webp",  # wrong size: the marker only renders the thumb crop
    ],
)
def test_events_schema_rejects_non_thumb_cdn_paths(image: str) -> None:
    """`image` must be a `/cdn/thumb/…` path: external URLs and other CDN sizes are rejected."""
    invalid = _valid_row()
    invalid["image"] = [image]
    df = pl.DataFrame(invalid, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(df)
