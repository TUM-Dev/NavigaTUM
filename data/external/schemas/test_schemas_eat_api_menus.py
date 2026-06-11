from datetime import date

import dataframely as dy
import polars as pl
import pytest

from external.schemas.eat_api_menus import EatApiMenuSchema
from external.scrapers.eat_api_menus import (
    _EatApiWeek,
    _iso_weeks,
    _rows_for_week,
)


def _valid_row() -> dict[str, list[object]]:
    """Build a single valid dish row in `EatApiMenuSchema` column order."""
    return {
        "canteen_id": ["mensa-garching"],
        "date": ["2026-06-10"],
        "position": [0],
        "name": ["Pasta Emiliana"],
        "dish_type": ["Pasta"],
        "prices_json": ['{"students":{"base_price":1.0,"price_per_unit":0.9,"unit":"100g"}}'],
        "labels_json": ['["GLUTEN","LACTOSE"]'],
        "source_url": ["https://tum-dev.github.io/eat-api/#!/de/mensa-garching"],
        "last_update": ["2026-06-05"],
    }


def _row_with(**overrides: object) -> pl.DataFrame:
    """Build a one-row frame from the valid baseline, overriding named columns."""
    row = _valid_row()
    for key, value in overrides.items():
        row[key] = [value]
    return pl.DataFrame(row, schema=EatApiMenuSchema.to_polars_schema())


def test_schema_accepts_minimal_valid_row() -> None:
    """A row matching every rule must validate cleanly (positive control)."""
    EatApiMenuSchema.validate(_row_with())


def test_schema_accepts_null_dish_type() -> None:
    """`dish_type` is upstream-optional and must accept null without complaint."""
    EatApiMenuSchema.validate(_row_with(dish_type=None))


def test_schema_rejects_empty_dish_type() -> None:
    """A whitespace-only `dish_type` must be rejected; null is the only "absent" signal."""
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(_row_with(dish_type="   "))


def test_schema_rejects_empty_name() -> None:
    """An empty `name` must be rejected; a dish without a title is never a real dish."""
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(_row_with(name=""))


def test_schema_rejects_negative_position() -> None:
    """`position` is a 0-based index; negatives are never valid."""
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(_row_with(position=-1))


def test_schema_rejects_duplicate_primary_key() -> None:
    """Two dishes with the same `(canteen_id, date, position)` would shadow each other."""
    duplicated = pl.DataFrame(
        {
            "canteen_id": ["mensa-garching"] * 2,
            "date": ["2026-06-10"] * 2,
            "position": [0, 0],
            "name": ["Pasta", "Suppe"],
            "dish_type": ["Pasta", "Suppe"],
            "prices_json": ["{}", "{}"],
            "labels_json": ["[]", "[]"],
            "source_url": ["https://x.tld"] * 2,
            "last_update": ["2026-06-05"] * 2,
        },
        schema=EatApiMenuSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(duplicated)


@pytest.mark.parametrize("bad_date", ["2026/06/10", "10-06-2026", "2026-6-1", "not-a-date"])
def test_schema_rejects_non_iso_date(bad_date: str) -> None:
    """`date` must be a zero-padded `YYYY-MM-DD` calendar date."""
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(_row_with(date=bad_date))


@pytest.mark.parametrize("bad_prices", ["null", "[]", '"x"', "1.0"])
def test_schema_rejects_non_object_prices_json(bad_prices: str) -> None:
    """`prices_json` must be a JSON object string; the response model expects a per-role mapping."""
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(_row_with(prices_json=bad_prices))


@pytest.mark.parametrize("bad_labels", ["{}", "null", '"GLUTEN"', "1"])
def test_schema_rejects_non_array_labels_json(bad_labels: str) -> None:
    """`labels_json` must be a JSON array string, even when empty (`[]`)."""
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(_row_with(labels_json=bad_labels))


@pytest.mark.parametrize("url", ["www.example.tld", "ftp://example.tld", "/relative", ""])
def test_schema_rejects_non_http_source_url(url: str) -> None:
    """`source_url` must be an absolute http(s) URL."""
    with pytest.raises(dy.exc.ValidationError):
        EatApiMenuSchema.validate(_row_with(source_url=url))


def test_iso_weeks_crosses_year_boundary() -> None:
    """The week sequence must follow ISO calendar rules across a year change."""
    # 2025-12-29 is ISO week 1 of 2026; the following Monday is week 2 of 2026.
    weeks = _iso_weeks(date(2025, 12, 29), count=2)
    assert weeks == [(2026, 1), (2026, 2)]


def test_iso_weeks_returns_requested_count_in_order() -> None:
    """The week sequence advances one ISO week at a time and never repeats."""
    weeks = _iso_weeks(date(2026, 6, 10), count=3)
    assert weeks == [(2026, 24), (2026, 25), (2026, 26)]


def test_rows_for_week_flattens_dishes_preserving_order() -> None:
    """Dishes flatten in upstream order; `position` is the source index within the day."""
    payload: _EatApiWeek = {
        "days": [
            {
                "date": "2026-06-10",
                "dishes": [
                    {"name": "Suppe", "dish_type": "Suppe", "prices": {}, "labels": []},
                    {"name": "Pasta", "dish_type": "Pasta", "prices": {}, "labels": ["GLUTEN"]},
                ],
            },
            {
                "date": "2026-06-11",
                "dishes": [
                    {"name": "Reisgericht", "prices": {"students": {"base_price": 2.0}}, "labels": []},
                ],
            },
        ]
    }
    rows = _rows_for_week("mensa-garching", payload, last_update="2026-06-05")
    assert [(row["date"], row["position"], row["name"]) for row in rows] == [
        ("2026-06-10", 0, "Suppe"),
        ("2026-06-10", 1, "Pasta"),
        ("2026-06-11", 0, "Reisgericht"),
    ]
    # A dish without `dish_type` should round-trip as null.
    assert rows[2]["dish_type"] is None


def test_rows_for_week_skips_dishes_without_a_name() -> None:
    """A blank `name` is never a real dish; we drop it rather than pollute the schema."""
    payload: _EatApiWeek = {
        "days": [
            {
                "date": "2026-06-10",
                "dishes": [
                    {"name": "  ", "prices": {}, "labels": []},
                    {"name": "Real Dish", "prices": {}, "labels": []},
                ],
            }
        ]
    }
    rows = _rows_for_week("mensa-garching", payload, last_update="2026-06-05")
    assert [row["name"] for row in rows] == ["Real Dish"]


def test_rows_for_week_handles_missing_days_block() -> None:
    """An eat-api response with no `days` (e.g. a closed-week stub) yields no rows, no error."""
    empty: _EatApiWeek = {"days": []}
    assert _rows_for_week("mensa-garching", empty, last_update="2026-06-05") == []
