import csv
import logging
from datetime import UTC, date, datetime, timedelta
from email.utils import parsedate_to_datetime
from pathlib import Path
from typing import NotRequired, TypedDict

import orjson
import polars as pl
import requests
from utils import setup_logging

from external.schemas.eat_api_menus import EatApiMenuSchema
from external.scraping_utils import CACHE_PATH


class _EatApiPrice(TypedDict, total=False):
    """One role's price block as eat-api emits it; every key is upstream-optional."""

    base_price: float
    price_per_unit: float
    unit: str


class _EatApiDish(TypedDict):
    """Single dish in eat-api's weekly JSON. `dish_type` is upstream-optional."""

    name: str
    dish_type: NotRequired[str | None]
    prices: dict[str, _EatApiPrice]
    labels: list[str]


class _EatApiDay(TypedDict):
    """One calendar day's dishes in eat-api's weekly JSON."""

    date: str
    dishes: list[_EatApiDish]


class _EatApiWeek(TypedDict):
    """Top-level weekly payload eat-api publishes per canteen."""

    days: list[_EatApiDay]


# eat-api stores one JSON per ISO week per canteen; we fetch the current and next ISO weeks so
# a Friday visitor still sees Monday. Going further out yields stale 404s on most canteens.
_WEEKS_AHEAD = 1
_WEEK_URL = "https://tum-dev.github.io/eat-api/{canteen_id}/{year}/{week:02d}.json"
_MENU_URL = "https://tum-dev.github.io/eat-api/#!/de/{canteen_id}"

# Same source file the processor consumes for the canteen-id -> NavigaTUM-id mapping. The
# scraper only needs the `canteen_id` column, so we read it directly rather than importing
# the processor and pulling in its dataframely dependency.
_CANTEEN_MAPPING_CSV = Path(__file__).resolve().parents[2] / "sources" / "mensa_canteens.csv"

_logger = logging.getLogger(__name__)


def _mapped_canteen_ids() -> list[str]:
    """Return the canteen ids the build is wired up to render menus for, in CSV order."""
    with _CANTEEN_MAPPING_CSV.open() as f:
        return [row["canteen_id"] for row in csv.DictReader(f)]


def _iso_weeks(reference: date, count: int) -> list[tuple[int, int]]:
    """Return `count` consecutive ISO `(year, week)` pairs starting at `reference`'s week."""
    weeks: list[tuple[int, int]] = []
    cursor = reference
    while len(weeks) < count:
        iso_year, iso_week, _ = cursor.isocalendar()
        weeks.append((iso_year, iso_week))
        # Step into next ISO week without depending on the calendar month.
        cursor += timedelta(days=7)
    return weeks


def _last_modified_date(response: requests.Response, fallback: date) -> str:
    """Return the response's `Last-Modified` as `YYYY-MM-DD`, falling back to `fallback`."""
    header = response.headers.get("Last-Modified")
    if header:
        return parsedate_to_datetime(header).date().isoformat()
    return fallback.isoformat()


def _fetch_week(canteen_id: str, year: int, week: int) -> tuple[_EatApiWeek | None, str | None]:
    """
    Fetch one week's menu for a canteen. Returns `(payload, last_modified_iso)`.

    A 404 means "no menu published for this week" - a real and common case for any canteen
    that is closed - and is treated as empty data, not an error.
    """
    url = _WEEK_URL.format(canteen_id=canteen_id, year=year, week=week)
    response = requests.get(url, timeout=30)
    if response.status_code == 404:
        return None, None
    response.raise_for_status()
    return response.json(), _last_modified_date(response, fallback=datetime.now(UTC).date())


def _rows_for_week(
    canteen_id: str,
    payload: _EatApiWeek,
    last_update: str,
) -> list[dict[str, object]]:
    """Flatten one week's payload into `EatApiMenuSchema` rows in serving order."""
    rows: list[dict[str, object]] = []
    for day in payload.get("days", []):
        day_date = day.get("date")
        if not day_date:
            continue
        for position, dish in enumerate(day.get("dishes", [])):
            name = (dish.get("name") or "").strip()
            if not name:
                continue
            rows.append(
                {
                    "canteen_id": canteen_id,
                    "date": day_date,
                    "position": position,
                    "name": name,
                    "dish_type": (dish.get("dish_type") or None),
                    "prices_json": orjson.dumps(dish.get("prices") or {}).decode(),
                    "labels_json": orjson.dumps(list(dish.get("labels") or [])).decode(),
                    "source_url": _MENU_URL.format(canteen_id=canteen_id),
                    "last_update": last_update,
                }
            )
    return rows


def scrape_eat_api_menus(*, today: date | None = None) -> None:
    """
    Write `eat_api_menus.csv` with the current and upcoming ISO week of menus per mapped canteen.

    Iterates the canteen ids in `mensa_canteens.csv`, since publishing a menu for a canteen
    NavigaTUM does not render would only widen the snapshot without benefit. Refuses to
    overwrite an existing CSV with an empty roster, so an upstream outage leaves stale (but
    correct) data in place rather than wiping coverage.
    """
    today = today or datetime.now(UTC).date()
    canteen_ids = _mapped_canteen_ids()
    weeks = _iso_weeks(today, count=1 + _WEEKS_AHEAD)

    all_rows: list[dict[str, object]] = []
    for canteen_id in canteen_ids:
        for year, week in weeks:
            payload, last_update = _fetch_week(canteen_id, year, week)
            if payload is None or last_update is None:
                _logger.info("no menu published for canteen %s in %s-W%02d", canteen_id, year, week)
                continue
            all_rows.extend(_rows_for_week(canteen_id, payload, last_update))

    if not all_rows:
        raise RuntimeError("eat-api returned no menus across the mapped canteens - refusing to overwrite the roster")

    df = pl.DataFrame(all_rows, schema=EatApiMenuSchema.to_polars_schema()).sort(["canteen_id", "date", "position"])
    EatApiMenuSchema.validate(df)
    df.write_csv(CACHE_PATH / "eat_api_menus.csv")
    _logger.info("Scraped %d dish rows across %d canteens", df.height, len(canteen_ids))


if __name__ == "__main__":
    setup_logging()
    CACHE_PATH.mkdir(exist_ok=True)
    scrape_eat_api_menus()
