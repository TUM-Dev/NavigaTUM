"""
Pure reference evaluator for an opening-hours schedule's live state.

The webclient computes *open now / closed / closes in X* from the OSM string itself;
this is the Python counterpart it mirrors, used to validate shipped schedules in tests.
`now` is always passed in (never `datetime.now()`) so results are reproducible. It wraps
`opening-hours-py`, a dev-only dependency, so it stays out of the runtime compile path.

The schedule must be plain OSM - macros and `PH` are expanded upstream
(`semester_block_expander`, `public_holiday_expander`). `now` is wall-clock in the
schedule's timezone (Europe/Berlin for TUM); pass an aware or naive datetime.
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Literal

import opening_hours


@dataclass(frozen=True)
class OpeningHoursState:
    """
    A schedule's live state at one instant.

    `until` is the end of the currently open interval (set only when open);
    `next_change` is the next opening time (set only when closed). Either is
    `None` when the schedule never changes again from `now` - a `24/7` place is
    open with no `until`, a place whose every interval is in the past is closed
    with no `next_change`.
    """

    state: Literal["open", "closed"]
    until: datetime | None
    next_change: datetime | None


def evaluate_state(schedule: str, now: datetime) -> OpeningHoursState:
    """Evaluate `schedule` (plain OSM) at `now`. See the module docstring for `now`'s timezone."""
    hours = opening_hours.OpeningHours(schedule)
    # `unknown` is not open; fold it into closed so callers see a binary state.
    kind, _comment = hours.state(now)
    is_open = kind == opening_hours.State.OPEN
    change = hours.next_change(now)
    if is_open:
        return OpeningHoursState(state="open", until=change, next_change=None)
    return OpeningHoursState(state="closed", until=None, next_change=change)
