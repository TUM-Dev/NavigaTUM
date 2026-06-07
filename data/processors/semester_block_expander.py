"""
The only place the `lecture:`/`break:` opening-hours macro dialect exists.

A schedule may prefix a rule block with `lecture:` (Vorlesungszeit) or `break:`
(vorlesungsfreie Zeit). `expand_semester_blocks` rewrites those into plain OSM
date ranges from an explicit semester list, so everything downstream sees
standard OSM. Keep the dialect isolated here so any OSM parser stays swappable.
"""

import re
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from datetime import date, timedelta

from external.schemas._validators import MACRO_REGEX

# Shares `MACRO_REGEX` with the on-disk schema rule so detection and expansion can't drift.
_MACRO_RE = re.compile(MACRO_REGEX)
# A single rule block's macro prefix and its OSM body.
_BLOCK_PREFIX_RE = re.compile(r"^(lecture|break)\s*:\s*(.*)$", re.IGNORECASE | re.DOTALL)
# OSM month abbreviations are English and locale-independent, so format them by hand.
_MONTHS = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")


@dataclass(frozen=True)
class Semester:
    """
    One academic semester: a calendar span containing a lecture sub-period.

    The break (vorlesungsfreie Zeit) is the calendar span outside
    `[lectures_from, lectures_until]` - the run-up before lectures and the tail after.
    """

    key: str
    start: date
    lectures_from: date
    lectures_until: date
    end: date

    @classmethod
    def from_row(cls, row: Mapping[str, object]) -> Semester:
        """Build a `Semester` from a validated `SemesterSchema` row (dates are `date` objects)."""
        return cls(
            key=str(row["key"]),
            start=_as_date(row["start"]),
            lectures_from=_as_date(row["lectures_from"]),
            lectures_until=_as_date(row["lectures_until"]),
            end=_as_date(row["end"]),
        )


def contains_macro(schedule: str) -> bool:
    """Whether `schedule` carries any `lecture:`/`break:` macro."""
    return _MACRO_RE.search(schedule) is not None


def expand_semester_blocks(schedule: str, semesters: Sequence[Semester]) -> str:
    """
    Expand `lecture:`/`break:` blocks into plain OSM date ranges, one per semester.

    Ordered by lecture start, and pure. A schedule with no macros is returned
    verbatim; an empty macro body (e.g. a bare `lecture:`) contributes no rule;
    plain-OSM blocks interleaved with macros are kept in place and apply
    unconditionally.
    """
    if not contains_macro(schedule):
        return schedule

    ordered = sorted(semesters, key=lambda semester: semester.lectures_from)
    rules: list[str] = []
    for raw_block in schedule.split(";"):
        block = raw_block.strip()
        if not block:
            continue
        match = _BLOCK_PREFIX_RE.match(block)
        if match is None:
            rules.append(block)  # plain OSM rule; applies unconditionally.
            continue
        kind, body = match.group(1).lower(), match.group(2).strip()
        if not body:
            continue  # a bare `lecture:` states nothing.
        for semester in ordered:
            rules.extend(f"{date_range} {body}" for date_range in _macro_ranges(kind, semester))
    return "; ".join(rules)


def _macro_ranges(kind: str, semester: Semester) -> list[str]:
    """OSM date ranges a `lecture:`/`break:` block maps to for one semester."""
    if kind == "lecture":
        return [_osm_range(semester.lectures_from, semester.lectures_until)]
    # `break:` is the non-lecture remainder: the run-up before lectures and the tail
    # after. Either is empty when a bound coincides with the lecture period.
    ranges: list[str] = []
    if semester.start < semester.lectures_from:
        ranges.append(_osm_range(semester.start, semester.lectures_from - timedelta(days=1)))
    if semester.lectures_until < semester.end:
        ranges.append(_osm_range(semester.lectures_until + timedelta(days=1), semester.end))
    return ranges


def _osm_range(start: date, end: date) -> str:
    """Format an OSM monthday range, e.g. `2025 Apr 22-2025 Aug 02`."""
    return f"{_osm_date(start)}-{_osm_date(end)}"


def _osm_date(day: date) -> str:
    """Format an OSM monthday selector, e.g. `2025 Apr 22` (zero-padded day, English month)."""
    return f"{day.year} {_MONTHS[day.month - 1]} {day.day:02d}"


def _as_date(value: object) -> date:
    """Narrow a `SemesterSchema` cell to `date` for the type checker (it always is)."""
    if not isinstance(value, date):
        raise TypeError(f"expected a date, got {type(value).__name__}: {value!r}")
    return value
