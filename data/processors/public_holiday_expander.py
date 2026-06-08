"""
The only place opening-hours `PH` (public-holiday) rules are turned into dates.

`opening-hours-py` resolves only *national* German holidays and the JS client has no
holiday calendar at all, so a `PH off` rule alone would stay open on Bavarian holidays
(Epiphany, Fronleichnam, Allerheiligen). `expand_public_holidays` bakes them in at
compile time instead, leaving self-contained plain OSM that every parser evaluates
identically. Kept isolated like `semester_block_expander` so any OSM parser stays swappable.
"""

import re
from collections.abc import Iterable, Sequence
from datetime import date

import holidays

# A standalone `PH` rule: the selector is exactly `PH`, optionally followed by a
# modifier or times (`off`, `closed`, `09:00-20:00`). Combined selectors such as
# `Mo-Fr,PH ...` are deliberately not matched - see `expand_public_holidays`.
_PH_RULE_RE = re.compile(r"^PH\b\s*(.*)$", re.DOTALL)
# Any `PH` token, to tell a combined selector apart from a schedule that has no PH.
_PH_TOKEN_RE = re.compile(r"\bPH\b")
# OSM month abbreviations are English and locale-independent, so format them by hand.
_MONTHS = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")


def contains_ph(schedule: str) -> bool:
    """Whether `schedule` carries any `PH` (public-holiday) selector."""
    return _PH_TOKEN_RE.search(schedule) is not None


def expand_public_holidays(schedule: str, holiday_dates: Sequence[date]) -> str:
    """
    Rewrite standalone `PH` rules into explicit per-date OSM rules, and return plain OSM.

    Each `PH <rest>` rule (e.g. `PH off`, `PH 09:00-12:00`) becomes one
    `<date> <rest>` rule per date in `holiday_dates`, appended after the regular
    rules so the holiday overrides the weekday schedule on that day. A schedule
    with no `PH` is returned verbatim; with no holidays the `PH` rule drops, as it
    would have no effect. Combined selectors (`Mo-Fr,PH off`) are rejected rather
    than risk a fragile rewrite.
    """
    if not contains_ph(schedule):
        return schedule

    kept: list[str] = []
    ph_bodies: list[str] = []
    for raw_block in schedule.split(";"):
        block = raw_block.strip()
        if not block:
            continue
        match = _PH_RULE_RE.match(block)
        if match is None:
            if _PH_TOKEN_RE.search(block):
                raise ValueError(
                    f"combined PH selector is not supported, split it into a standalone `PH` rule: {block!r}"
                )
            kept.append(block)  # plain OSM rule; carried through untouched.
            continue
        ph_bodies.append(match.group(1).strip())

    dates = sorted(set(holiday_dates))
    overrides = [f"{_osm_date(day)} {body}".strip() for body in ph_bodies for day in dates]
    return "; ".join(kept + overrides)


def bavarian_holiday_dates(years: Iterable[int]) -> list[date]:
    """Bavarian (`DE-BY`) public-holiday dates across `years`, the source for `PH` expansion."""
    calendar = holidays.country_holidays("DE", subdiv="BY", years=sorted(set(years)))
    return sorted(calendar.keys())


def _osm_date(day: date) -> str:
    """Format an OSM monthday selector, e.g. `2026 Jan 06` (zero-padded day, English month)."""
    return f"{day.year} {_MONTHS[day.month - 1]} {day.day:02d}"
