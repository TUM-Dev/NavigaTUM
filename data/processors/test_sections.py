from typing import TypedDict

import orjson
import polars as pl

from processors.sections import generate_buildings_overview


class LookupRow(TypedDict):
    """One node as `generate_buildings_overview` reads it from the export frame."""

    id: str
    type: str
    name: str
    short_name: str | None
    props_stats_n_rooms: int
    props_stats_n_buildings: int
    imgs_json: str | None
    children_flat: list[str] | None
    children: list[str] | None
    generators_json: str | None
    sections_buildings_overview_json: str | None


class OverviewEntry(TypedDict):
    """One decoded entry of a parent's `buildings_overview` section."""

    id: str
    type: str
    name: str
    subtext: str
    thumb: str | None


def _lookup_row(
    _id: str,
    _type: str,
    name: str,
    *,
    n_rooms: int = 0,
    n_buildings: int = 0,
    children: list[str] | None = None,
    children_flat: list[str] | None = None,
) -> LookupRow:
    """One node as `generate_buildings_overview` expects it in the global lookup."""
    return {
        "id": _id,
        "type": _type,
        "name": name,
        "short_name": None,
        "props_stats_n_rooms": n_rooms,
        "props_stats_n_buildings": n_buildings,
        "imgs_json": None,
        "children_flat": children_flat,
        "children": children,
        "generators_json": None,
        "sections_buildings_overview_json": None,
    }


def _overview_entries(df: pl.DataFrame, parent_id: str) -> list[OverviewEntry]:
    """Run the generator and decode the parent's resulting overview entries."""
    result = generate_buildings_overview(df)
    raw = result.filter(pl.col("id") == parent_id)["sections_buildings_overview_json"].item()
    entries: list[OverviewEntry] = orjson.loads(raw)["entries"]
    return entries


def test_buildings_overview_entries_carry_child_type() -> None:
    """
    Each overview entry exposes the child's entity `type`.

    A campus lists children of mixed types (building, area, site). The frontend routes
    them to type-specific canonical paths (area/site -> /site, building -> /building), so
    the type must travel with each entry rather than being inferred client-side.
    """
    children = ["b1", "a1", "s1"]
    df = pl.DataFrame(
        [
            _lookup_row("root", "campus", "Root", children=children, children_flat=children),
            _lookup_row("b1", "building", "Bravo", n_rooms=5),
            _lookup_row("a1", "area", "Alpha", n_rooms=100, n_buildings=3),
            _lookup_row("s1", "site", "Sierra", n_rooms=50, n_buildings=2),
        ],
    )

    entries = _overview_entries(df, "root")

    types_by_id = {e["id"]: e["type"] for e in entries}
    assert types_by_id == {"b1": "building", "a1": "area", "s1": "site"}
