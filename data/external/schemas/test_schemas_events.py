import dataframely as dy
import polars as pl
import pytest

from external.loaders.events import load_events
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.events import EventsSchema


def _valid_row() -> dict[str, list[object]]:
    return {
        "image": ["/cdn/thumb/abc123def4567890_0.webp"],
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


@pytest.mark.parametrize(
    "image",
    [
        "https://www.tum.de/fileadmin/_processed_/c/5/csm_1584780_0346e6dc92.webp",  # external host
        "2a2e032bb328fa01_0.webp",  # bare filename, missing the /cdn/thumb/ delivery prefix
        "/cdn/lg/2a2e032bb328fa01_0.webp",  # wrong size: the marker only renders the thumb crop
    ],
)
def test_events_schema_rejects_non_thumb_cdn_paths(image: str) -> None:
    """`image` must be a `/cdn/thumb/…` path: external URLs and other CDN sizes are rejected."""
    invalid = _valid_row()
    invalid["image"] = [image]
    df = pl.DataFrame(invalid, schema=EventsSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        EventsSchema.validate(df)
