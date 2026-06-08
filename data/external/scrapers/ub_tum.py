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

INDEX_URL = "https://www.ub.tum.de/en/branch-libraries"
_BRANCH_HREF_RE = re.compile(r"^/en/branch-library-([a-z0-9-]+)$")

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
# Drupal emits en/em dashes in slot ranges, but OSM only accepts ASCII hyphen.
_HYPHEN_LIKE = ("\u2013", "\u2014")

# Drupal `field__label` text mapped to a semester macro.
# Empty string means "no macro", any label absent from the table is treated as a service variant.
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
    branch_id: str
    name: str
    opening_hours: str
    source_url: str


def parse_branch_page(html: str, *, source_url: str, branch_id: str, name: str) -> ParsedBranchHours:
    """Parse a ub.tum.de branch-library page into one ParsedBranchHours."""
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
    label = paragraph.select_one("div.field__label")
    return label.get_text(strip=True) if label else ""


def _classify_label(label: str) -> tuple[str, str]:
    """Return `(macro_prefix, trailing_comment)` for a paragraph label."""
    key = label.strip().lower()
    if key in _SEASON_LABELS:
        return _SEASON_LABELS[key], ""
    return "", label.strip()


def _day_slots(paragraph: Tag) -> list[str | None]:
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
    cleaned = raw
    for dash in _HYPHEN_LIKE:
        cleaned = cleaned.replace(dash, "-")
    cleaned = cleaned.strip()
    if not cleaned or cleaned.lower() in _CLOSED_TOKENS:
        return None
    parts = [_normalize_range(part) for part in cleaned.split(",")]
    return ",".join(filter(None, parts)) or None


def _normalize_range(part: str) -> str:
    start, _, end = part.partition("-")
    return f"{_pad(start)}-{_pad(end)}"


def _pad(time: str) -> str:
    cleaned = time.strip()
    hour, _, minute = cleaned.partition(":")
    return f"{int(hour):02d}:{minute or '00'}"


@dataclass
class _DayRun:
    first_abbr: str
    last_abbr: str
    last_index: int
    slot: str

    def day_range(self) -> str:
        return self.first_abbr if self.first_abbr == self.last_abbr else f"{self.first_abbr}-{self.last_abbr}"


def _day_runs(slots: list[str | None]) -> list[tuple[str, str]]:
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
    body = f"{day_range} {slot}"
    if comment:
        body = f'{body} "{comment}"'
    if prefix:
        return f"{prefix}: {body}"
    return body


def _discover_branches(session: requests.Session) -> list[tuple[str, str]]:
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
    soup = BeautifulSoup(html, "lxml")
    heading = soup.select_one("div.views-field-name .field-content")
    if heading is None:
        return fallback
    text = heading.get_text(" ", strip=True)
    return text or fallback


def scrape_ub_tum() -> None:
    """Write ub_tum.csv from the live ub.tum.de branch-library pages."""
    # Refuse to overwrite with an empty roster on a transient outage or layout change (cf. #1087).
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
