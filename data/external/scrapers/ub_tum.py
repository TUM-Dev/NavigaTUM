import logging
import re
from dataclasses import dataclass
from datetime import UTC, datetime
from urllib.parse import urljoin

import polars as pl
import requests
from bs4 import BeautifulSoup, Tag
from utils import setup_logging

from external.schemas.ub_tum import UbTumSchema
from external.scraping_utils import CACHE_PATH

# Index page that lists every Teilbibliothek (English: branch library); the slug between
# `/en/branch-library-` and the end of the URL keys the branch in the cache
# (e.g. `mathematics-informatics`).
INDEX_URL = "https://www.ub.tum.de/en/branch-libraries"
_BRANCH_HREF_RE = re.compile(r"^/en/branch-library-([a-z0-9-]+)$")

# English weekday label -> OSM abbreviation, in calendar order.
_WEEKDAYS: list[tuple[str, str]] = [
    ("Monday", "Mo"),
    ("Tuesday", "Tu"),
    ("Wednesday", "We"),
    ("Thursday", "Th"),
    ("Friday", "Fr"),
    ("Saturday", "Sa"),
    ("Sunday", "Su"),
]
_DAY_BY_LABEL: dict[str, tuple[int, str]] = {
    label.lower(): (index, abbreviation) for index, (label, abbreviation) in enumerate(_WEEKDAYS)
}
_CLOSED_TOKENS = frozenset({"closed", "geschlossen"})
# Drupal sometimes substitutes an en-dash or em-dash for the ASCII hyphen-minus in slot
# ranges; OSM only accepts the ASCII form, so both upstream variants are normalised away.
_HYPHEN_LIKE = ("\u2013", "\u2014")

# Drupal office_hours `field__label` text -> canonical block shape. A paragraph
# whose label matches a season-key emits its rules under that semester macro;
# anything not in the table is treated as a service variant whose label becomes
# a per-rule trailing OSM comment. "Zeiten" is the Drupal field default and means
# "year-round, no variant".
_SEASON_LABELS: dict[str, str] = {
    "zeiten": "",
    "times": "",
    "hours": "",
    "opening hours": "",
    "vorlesungszeit": "lecture",
    "lecture period": "lecture",
    "vorlesungsfreie zeit": "break",
    "vorlesungsfrei": "break",
    "semester break": "break",
    "semesterferien": "break",
}

_logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class ParsedBranchHours:
    """One UB-TUM branch parsed from its HTML page."""

    branch_id: str
    name: str
    opening_hours: str
    source_url: str


def parse_branch_page(html: str, *, source_url: str, branch_id: str, name: str) -> ParsedBranchHours:
    """
    Parse a `ub.tum.de` branch-library page into one `ParsedBranchHours`.

    Walks every `paragraph--type--oeffnungszeiten` block. The block's `field__label`
    picks one of three shapes per the `_SEASON_LABELS` table: a default block emits
    plain rules; a `lecture:`/`break:` season block prefixes each rule with that
    macro; any other label is treated as a service variant and becomes a per-rule
    trailing OSM comment (e.g. `Mo-Fr 09:00-20:00 "Pickup of preordered books"`).
    """
    soup = BeautifulSoup(html, "lxml")
    paragraphs = soup.select("div.paragraph--type--oeffnungszeiten")
    if not paragraphs:
        raise ValueError(f"branch page {source_url!r} has no opening-hours paragraph")

    rules: list[str] = []
    for paragraph in paragraphs:
        prefix, comment = _classify_label(_field_label_text(paragraph))
        for day_range, slot in _day_runs(_day_slots(paragraph)):
            rules.append(_format_rule(day_range, slot, prefix=prefix, comment=comment))

    opening_hours = "; ".join(rules)
    if not opening_hours:
        raise ValueError(f"branch page {source_url!r} produced no opening-hours rules")
    return ParsedBranchHours(branch_id=branch_id, name=name, opening_hours=opening_hours, source_url=source_url)


def _field_label_text(paragraph: Tag) -> str:
    """Return the `field__label` div text inside an opening-hours paragraph; empty when absent."""
    label = paragraph.select_one("div.field__label")
    return label.get_text(strip=True) if label else ""


def _classify_label(label: str) -> tuple[str, str]:
    """Return `(macro_prefix, trailing_comment)` for a paragraph label."""
    key = label.strip().lower()
    if key in _SEASON_LABELS:
        return _SEASON_LABELS[key], ""
    # Unknown label means this paragraph describes a service variant; the label
    # itself becomes the per-rule trailing comment so the renderer can group on it.
    return "", label.strip()


def _day_slots(paragraph: Tag) -> list[str | None]:
    """
    Return the 7 daily slots in calendar order, `None` for a closed day.

    The Drupal office_hours markup emits one `office-hours__item` per weekday with
    a day-name label span and a slot span; days not present in the markup default
    to closed. Multiple ranges within a single day collapse to a comma-separated
    OSM list.
    """
    slots: list[str | None] = [None] * len(_WEEKDAYS)
    for item in paragraph.select("div.office-hours__item"):
        label_el = item.select_one("span.office-hours__item-label")
        slot_el = item.select_one("span.office-hours__item-slots")
        if label_el is None or slot_el is None:
            continue
        day_text = label_el.get_text(strip=True).rstrip(":").strip().lower()
        position = _DAY_BY_LABEL.get(day_text)
        if position is None:
            continue
        index, _ = position
        slots[index] = _normalize_slot(slot_el.get_text(" ", strip=True))
    return slots


def _normalize_slot(raw: str) -> str | None:
    """Normalise a slot cell to an OSM range list, or `None` for a closed day."""
    cleaned = raw
    for dash in _HYPHEN_LIKE:
        cleaned = cleaned.replace(dash, "-")
    cleaned = cleaned.strip()
    if not cleaned or cleaned.lower() in _CLOSED_TOKENS:
        return None
    parts = [_normalize_range(part) for part in cleaned.split(",")]
    return ",".join(filter(None, parts)) or None


def _normalize_range(part: str) -> str:
    """Zero-pad the hour fields in a `H:MM-H:MM` range so the result parses as OSM."""
    start, _, end = part.partition("-")
    return f"{_pad(start)}-{_pad(end)}"


def _pad(time: str) -> str:
    """Zero-pad a `H:MM` time to `HH:MM` (leaves `HH:MM` untouched)."""
    cleaned = time.strip()
    hour, _, minute = cleaned.partition(":")
    return f"{int(hour):02d}:{minute or '00'}"


@dataclass
class _DayRun:
    """A run of consecutive weekdays sharing a slot, rendered as one OSM rule."""

    first_abbr: str
    last_abbr: str
    last_index: int
    slot: str

    def day_range(self) -> str:
        return self.first_abbr if self.first_abbr == self.last_abbr else f"{self.first_abbr}-{self.last_abbr}"


def _day_runs(slots: list[str | None]) -> list[tuple[str, str]]:
    """Collapse calendar-ordered slots into `(day_range, slot)` pairs, skipping closed days."""
    runs: list[_DayRun] = []
    for index, (_, abbreviation) in enumerate(_WEEKDAYS):
        slot = slots[index]
        if slot is None:
            continue
        previous = runs[-1] if runs else None
        if previous and previous.last_index == index - 1 and previous.slot == slot:
            previous.last_abbr = abbreviation
            previous.last_index = index
        else:
            runs.append(_DayRun(abbreviation, abbreviation, index, slot))
    return [(run.day_range(), run.slot) for run in runs]


def _format_rule(day_range: str, slot: str, *, prefix: str, comment: str) -> str:
    """Format one OSM rule: optional `lecture:`/`break:` prefix, the body, optional trailing comment."""
    body = f"{day_range} {slot}"
    if comment:
        body = f'{body} "{comment}"'
    if prefix:
        return f"{prefix}: {body}"
    return body


def _discover_branches(session: requests.Session) -> list[tuple[str, str]]:
    """Return `(branch_id, source_url)` pairs from the branch-libraries index."""
    response = session.get(INDEX_URL, timeout=30)
    response.raise_for_status()
    soup = BeautifulSoup(response.text, "lxml")
    seen: dict[str, str] = {}
    for anchor in soup.select("a[href]"):
        href = anchor.get("href")
        if not isinstance(href, str):
            continue
        match = _BRANCH_HREF_RE.match(href)
        if match is None:
            continue
        branch_id = match.group(1)
        seen.setdefault(branch_id, urljoin(INDEX_URL, href))
    if not seen:
        raise RuntimeError(f"branch-libraries index {INDEX_URL!r} listed no branch sub-pages")
    return sorted(seen.items())


def _branch_name(html: str, *, fallback: str) -> str:
    """Pull the per-branch heading as the name; fall back to the slug if absent."""
    soup = BeautifulSoup(html, "lxml")
    heading = soup.select_one("div.views-field-name .field-content")
    if heading is None:
        return fallback
    text = heading.get_text(" ", strip=True)
    return text or fallback


def scrape_ub_tum() -> None:
    """
    Write `ub_tum.csv` from the live `ub.tum.de` branch-library pages.

    Refuses to overwrite with an empty roster, so a transient outage or upstream
    layout change that drops every paragraph leaves the previously scraped data in
    place rather than wiping coverage (cf. #1087).
    """
    session = requests.Session()
    today = datetime.now(tz=UTC).date().isoformat()

    rows: list[dict[str, str]] = []
    for branch_id, source_url in _discover_branches(session):
        response = session.get(source_url, timeout=30)
        response.raise_for_status()
        name = _branch_name(response.text, fallback=branch_id)
        try:
            parsed = parse_branch_page(response.text, source_url=source_url, branch_id=branch_id, name=name)
        except ValueError:
            _logger.exception("failed to parse %s; skipping", source_url)
            continue
        rows.append(
            {
                "branch_id": parsed.branch_id,
                "name": parsed.name,
                "opening_hours": parsed.opening_hours,
                "last_update": today,
                "source_url": parsed.source_url,
            }
        )
    if not rows:
        raise RuntimeError("ub.tum.de produced no parseable branch pages - refusing to overwrite the roster")

    df = pl.DataFrame(rows, schema=UbTumSchema.to_polars_schema()).sort("branch_id")
    UbTumSchema.validate(df)
    df.write_csv(CACHE_PATH / "ub_tum.csv")
    _logger.info(f"Scraped opening hours for {df.height} UB-TUM branches")


if __name__ == "__main__":
    setup_logging()
    CACHE_PATH.mkdir(exist_ok=True)
    scrape_ub_tum()
