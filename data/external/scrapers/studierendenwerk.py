import logging
from dataclasses import dataclass
from datetime import UTC, datetime
from email.utils import parsedate_to_datetime

import polars as pl
import requests
from utils import setup_logging

from external.schemas.studierendenwerk import StudierendenwerkSchema
from external.scraping_utils import CACHE_PATH

# Studierendenwerk München publishes no structured feed; eat-api scrapes its HTML and
# exposes the result as MIT-licensed JSON, so it is the source rather than re-scraping HTML.
CANTEENS_URL = "https://tum-dev.github.io/eat-api/enums/canteens.json"
# Human-facing menu page per canteen, mirroring the existing Speiseplan links in links.yaml.
_MENU_URL = "https://tum-dev.github.io/eat-api/#!/de/{canteen_id}"

# eat-api weekday keys in calendar order, paired with their OSM weekday abbreviations.
_WEEKDAYS: list[tuple[str, str]] = [
    ("mon", "Mo"),
    ("tue", "Tu"),
    ("wed", "We"),
    ("thu", "Th"),
    ("fri", "Fr"),
    ("sat", "Sa"),
    ("sun", "Su"),
]

_logger = logging.getLogger(__name__)


@dataclass
class _DayGroup:
    """A run of consecutive weekdays sharing the same start/end, as one OSM rule."""

    first_abbr: str
    last_abbr: str
    last_index: int
    start: str
    end: str

    def to_osm(self) -> str:
        days = self.first_abbr if self.first_abbr == self.last_abbr else f"{self.first_abbr}-{self.last_abbr}"
        return f"{days} {self.start}-{self.end}"


def open_hours_to_osm(open_hours: dict[str, dict[str, str]]) -> str:
    """
    Convert eat-api per-weekday `open_hours` into an OSM `opening_hours` string.

    Consecutive weekdays sharing the same start/end collapse into one OSM day range
    (e.g. `Mo-Fr 11:00-14:00`); a differing or closed day starts a new rule, so a gap
    never produces a range that silently includes a closed day. Returns `""` when no
    day is open.
    """
    groups: list[_DayGroup] = []
    for index, (key, abbreviation) in enumerate(_WEEKDAYS):
        slot = open_hours.get(key)
        if not slot:
            continue
        start, end = slot["start"], slot["end"]
        previous = groups[-1] if groups else None
        if previous and previous.last_index == index - 1 and (previous.start, previous.end) == (start, end):
            previous.last_abbr = abbreviation
            previous.last_index = index
        else:
            groups.append(_DayGroup(abbreviation, abbreviation, index, start, end))

    return "; ".join(group.to_osm() for group in groups)


def _last_modified_date(response: requests.Response) -> str:
    """Return the feed's `Last-Modified` as a `YYYY-MM-DD` date, falling back to today (UTC)."""
    header = response.headers.get("Last-Modified")
    if header:
        return parsedate_to_datetime(header).date().isoformat()
    return datetime.now(UTC).date().isoformat()


def scrape_studierendenwerk() -> None:
    """
    Write `studierendenwerk.csv` from the live eat-api canteen feed.

    Refuses to overwrite with an empty roster, so a transient outage or an upstream
    layout change that drops all opening hours leaves the previously scraped data in
    place rather than wiping coverage.
    """
    response = requests.get(CANTEENS_URL, timeout=30)
    response.raise_for_status()
    last_update = _last_modified_date(response)

    rows = []
    for canteen in response.json():
        opening_hours = open_hours_to_osm(canteen.get("open_hours") or {})
        if not opening_hours:
            continue
        rows.append(
            {
                "canteen_id": canteen["canteen_id"],
                "name": canteen["name"],
                "opening_hours": opening_hours,
                "last_update": last_update,
                "source_url": _MENU_URL.format(canteen_id=canteen["canteen_id"]),
            }
        )
    if not rows:
        raise RuntimeError("eat-api returned no canteens with opening hours - refusing to overwrite the roster")

    df = pl.DataFrame(rows, schema=StudierendenwerkSchema.to_polars_schema()).sort("canteen_id")
    StudierendenwerkSchema.validate(df)
    df.write_csv(CACHE_PATH / "studierendenwerk.csv")
    _logger.info(f"Scraped opening hours for {df.height} canteens")


if __name__ == "__main__":
    setup_logging()
    CACHE_PATH.mkdir(exist_ok=True)
    scrape_studierendenwerk()
