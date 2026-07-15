"""Regression tests for the /search export document (TUM-Dev/NavigaTUM#3399)."""

from pathlib import Path

import polars as pl
import pytest
from pipeline_types import Entry
from utils import TranslatableStr

import processors.export as export_mod
from processors.export import export_for_search
from processors.images import ImageSource


def _data() -> dict[str, Entry]:
    """Minimal chain root -> garching -> 5204 (short_name UTG), mirroring the real areatree line."""
    return {
        "root": {"id": "root", "type": "root", "name": TranslatableStr("Standorte", "Sites"), "parents": []},
        "garching": {
            "id": "garching",
            "type": "site",
            "name": TranslatableStr("Garching", "Garching"),
            "type_common_name": TranslatableStr("Standort", "Site"),
            "parents": ["root"],
            "ranking_factors": {"rank_combined": 100},
        },
        "5204": {
            "id": "5204",
            "type": "building",
            "name": TranslatableStr("Umformtechnik und Gießereiwesen (MW25)", "Metal Forming and Casting"),
            "short_name": "UTG",
            "visible_id": "utg",
            "type_common_name": TranslatableStr("Gebäude", "Building"),
            "parents": ["root", "garching"],
            "ranking_factors": {"rank_combined": 100},
        },
    }


def _search_docs(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> pl.DataFrame:
    """Run the real search export against `_data()` in isolation and read back the documents."""
    monkeypatch.setattr(export_mod, "OUTPUT_DIR_PATH", tmp_path)
    monkeypatch.setattr(export_mod, "load_events", list)
    monkeypatch.setattr(export_mod, "event_search_documents", lambda *_a, **_k: [])
    monkeypatch.setattr(ImageSource, "load_all", classmethod(lambda _cls: {}))
    export_for_search(_data())
    return pl.read_parquet(tmp_path / "search_data.parquet")


def test_building_own_short_name_is_indexed(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """A building's own short_name reaches its search document, so `utg` finds building 5204 itself (#3399)."""
    df = _search_docs(tmp_path, monkeypatch)
    building = next(r for r in df.to_dicts() if r["room_code"] == "5204")

    assert building["short_name"] == "UTG"


def test_entries_without_a_short_name_index_null(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Entries lacking a short_name emit null (not an empty string), matching other optional fields."""
    df = _search_docs(tmp_path, monkeypatch)
    garching = next(r for r in df.to_dicts() if r["room_code"] == "garching")

    assert garching["short_name"] is None
