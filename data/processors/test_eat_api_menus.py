import logging
from datetime import date

import orjson
import polars as pl
import pytest
from external.schemas.eat_api_menus import EatApiMenuSchema

from processors.eat_api_menus import merge_mensa_menus

_TODAY = date(2026, 6, 10)
_SOURCE = "https://tum-dev.github.io/eat-api/#!/de/mensa-garching"


def _menus(**overrides: list[object]) -> pl.DataFrame:
    """Build a two-day, two-dish menu feed matching `EatApiMenuSchema`."""
    base: dict[str, list[object]] = {
        "canteen_id": ["mensa-garching", "mensa-garching", "mensa-garching"],
        "date": ["2026-06-10", "2026-06-10", "2026-06-11"],
        "position": [0, 1, 0],
        "name": ["Suppe", "Pasta", "Reisgericht"],
        "dish_type": ["Suppe", "Pasta", None],
        "prices_json": [
            "{}",
            '{"students":{"base_price":1.0,"price_per_unit":0.9,"unit":"100g"}}',
            "{}",
        ],
        "labels_json": ["[]", '["GLUTEN","LACTOSE"]', "[]"],
        "source_url": [_SOURCE, _SOURCE, _SOURCE],
        "last_update": ["2026-06-05", "2026-06-05", "2026-06-05"],
    }
    return pl.DataFrame({**base, **overrides}, schema=EatApiMenuSchema.to_polars_schema())


def _mapping(canteen_ids: list[str], ids: list[str]) -> pl.DataFrame:
    return pl.DataFrame({"canteen_id": canteen_ids, "id": ids}, schema={"canteen_id": pl.Utf8(), "id": pl.Utf8()})


def _entries(ids: list[str]) -> pl.DataFrame:
    return pl.DataFrame({"id": ids, "type": ["building"] * len(ids)})


def test_attaches_menu_payload_to_mapped_entry() -> None:
    """A mapped canteen yields a serialized `MenuResponse`-shaped payload on its NavigaTUM id."""
    entries = _entries(["5304", "0001"])
    merged = merge_mensa_menus(
        entries,
        menus=_menus(),
        mapping=_mapping(["mensa-garching"], ["5304"]),
        today=_TODAY,
    )
    by_id = dict(zip(merged["id"], merged["mensa_menus_json"], strict=True))
    assert by_id["0001"] is None
    payload = orjson.loads(by_id["5304"])
    assert payload["source_url"] == _SOURCE
    assert payload["last_update"] == "2026-06-05"
    assert [day["date"] for day in payload["days"]] == ["2026-06-10", "2026-06-11"]
    monday_dishes = payload["days"][0]["dishes"]
    assert [dish["name"] for dish in monday_dishes] == ["Suppe", "Pasta"]
    pasta = monday_dishes[1]
    assert pasta["dish_type"] == "Pasta"
    assert pasta["prices"]["students"]["price_per_unit"] == 0.9
    assert pasta["labels"] == ["GLUTEN", "LACTOSE"]


def test_omits_dish_type_when_upstream_null() -> None:
    """`dish_type` is skipped on dishes where upstream did not supply it; null bloats the payload."""
    entries = _entries(["5304"])
    merged = merge_mensa_menus(
        entries,
        menus=_menus(),
        mapping=_mapping(["mensa-garching"], ["5304"]),
        today=_TODAY,
    )
    payload = orjson.loads(merged["mensa_menus_json"][0])
    tuesday = payload["days"][1]["dishes"][0]
    assert "dish_type" not in tuesday


def test_warns_when_mapping_targets_canteen_absent_from_feed(caplog: pytest.LogCaptureFixture) -> None:
    """A mapping entry with no matching feed canteen is logged and produces no payload."""
    entries = _entries(["5304", "9999"])
    with caplog.at_level(logging.WARNING):
        merged = merge_mensa_menus(
            entries,
            menus=_menus(),
            mapping=_mapping(["mensa-garching", "mensa-gone"], ["5304", "9999"]),
            today=_TODAY,
        )
    by_id = dict(zip(merged["id"], merged["mensa_menus_json"], strict=True))
    assert by_id["5304"] is not None
    assert by_id["9999"] is None
    assert "mensa-gone" in caplog.text


def test_warns_when_feed_is_stale(caplog: pytest.LogCaptureFixture) -> None:
    """A feed snapshot older than the staleness window is flagged at build time."""
    entries = _entries(["5304"])
    with caplog.at_level(logging.WARNING):
        merge_mensa_menus(
            entries,
            menus=_menus(last_update=["2025-01-01", "2025-01-01", "2025-01-01"]),
            mapping=_mapping(["mensa-garching"], ["5304"]),
            today=_TODAY,
        )
    assert "stale" in caplog.text


def test_returns_unchanged_frame_when_no_canteens_match() -> None:
    """An empty join must not introduce a `mensa_menus_json` column on unrelated builds."""
    entries = _entries(["0001"])
    merged = merge_mensa_menus(
        entries,
        menus=_menus(),
        mapping=_mapping(["mensa-elsewhere"], ["9999"]),
        today=_TODAY,
    )
    assert "mensa_menus_json" not in merged.columns


def test_fails_when_payload_targets_unknown_entry_id() -> None:
    """A mapping pointing at an id absent from the entries frame is a build-time error."""
    entries = _entries(["0001"])
    with pytest.raises(ValueError, match="unknown entry id"):
        merge_mensa_menus(
            entries,
            menus=_menus(),
            mapping=_mapping(["mensa-garching"], ["5304"]),
            today=_TODAY,
        )
