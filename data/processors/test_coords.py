import polars as pl
import pytest

from processors.coords import validate_coords


def test_validate_coords_allows_room_403m_from_parent() -> None:
    """A sports field can be slightly farther from the parent coordinate than the usual limit."""
    df = pl.DataFrame(
        [
            {
                "id": "2321",
                "type": "building",
                "coords_lat": 48.18140110956286,
                "coords_lon": 11.541902400077351,
                "parents": ["root"],
            },
            {
                "id": "2321.EG.128",
                "type": "room",
                "coords_lat": 48.17855501242107,
                "coords_lon": 11.54527411512356,
                "parents": ["root", "2321"],
            },
        ]
    )

    validate_coords(df)


def test_validate_coords_rejects_rooms_more_than_450m_from_parent() -> None:
    """The relaxed limit still rejects rooms implausibly far from their parent."""
    df = pl.DataFrame(
        [
            {
                "id": "2321",
                "type": "building",
                "coords_lat": 48.18140110956286,
                "coords_lon": 11.541902400077351,
                "parents": ["root"],
            },
            {
                "id": "far-room",
                "type": "room",
                "coords_lat": 48.175,
                "coords_lon": 11.54527411512356,
                "parents": ["root", "2321"],
            },
        ]
    )

    with pytest.raises(RuntimeError, match="far-room is"):
        validate_coords(df)
