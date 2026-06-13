"""
Compute an event's map lead-in: when its marker should first appear.

`appears_at` answers "this is worth surfacing now" by walking backward from
`starts_at`, spending a fixed budget at a traffic rate that drains slower when
fewer people look at the map (nights, weekends), so quiet-time events ride
longer. It is a pure function of `(starts_at, ends_at)`, evaluated in
Europe/Berlin wall-clock time and DST-aware, and is consumed only as a
server-side visibility gate - it never reaches the client.
"""

from datetime import UTC, datetime, timedelta
from zoneinfo import ZoneInfo

_BERLIN = ZoneInfo("Europe/Berlin")

_BUDGET_FLOOR_HOURS = 8.0
_WINDOW_CAP_HOURS = 48.0
# Wall-clock hours bounding weekday daytime, the baseline (fastest-draining) rate.
_DAY_START_HOUR = 10
_DAY_END_HOUR = 18


def _traffic_rate(local: datetime) -> float:
    """
    Budget drained per real hour at Berlin wall-clock `local`.

    1.0 is the weekday-daytime baseline; quieter periods drain slower so the
    marker rides longer. Night (0.5) and weekend (0.5) stack multiplicatively,
    so a weekend night is 0.25.
    """
    rate = 1.0 if _DAY_START_HOUR <= local.hour < _DAY_END_HOUR else 0.5
    if local.weekday() >= 5:  # Saturday or Sunday.
        rate *= 0.5
    return rate


def _previous_boundary(local: datetime) -> datetime:
    """
    Return the latest traffic-rate boundary strictly before `local` (Berlin wall-clock).

    The rate only changes at 00:00, 10:00 and 18:00, so the half-open segment
    `[boundary, local)` carries one constant rate. Classifying by the boundary
    rather than `local` keeps a Friday-night segment a weekday when it is reached
    by walking back through Saturday 00:00.
    """
    midnight = local.replace(hour=0, minute=0, second=0, microsecond=0)
    for hour in (_DAY_END_HOUR, _DAY_START_HOUR, 0):
        boundary = midnight.replace(hour=hour)
        if boundary < local:
            return boundary
    # `local` is exactly midnight: step back to 18:00 the previous day.
    return (midnight - timedelta(days=1)).replace(hour=_DAY_END_HOUR)


def compute_appears_at(starts_at: datetime, ends_at: datetime) -> datetime:
    """
    When the event's marker should first appear, returned in Europe/Berlin time.

    `starts_at`/`ends_at` must be timezone-aware; the result is the same instant
    whatever input offset is used. See the module docstring for the algorithm.
    """
    if starts_at.tzinfo is None or ends_at.tzinfo is None:
        raise ValueError("compute_appears_at requires timezone-aware datetimes")

    duration_hours = (ends_at - starts_at).total_seconds() / 3600
    budget = max(duration_hours, _BUDGET_FLOOR_HOURS)

    # Walk in UTC so timedelta steps are real elapsed time across DST; Berlin only classifies the rate.
    instant = starts_at.astimezone(UTC)
    cap = instant - timedelta(hours=_WINDOW_CAP_HOURS)
    while budget > 1e-9 and instant > cap:
        boundary_local = _previous_boundary(instant.astimezone(_BERLIN))
        rate = _traffic_rate(boundary_local)
        segment_start = max(boundary_local.astimezone(UTC), cap)
        cost = rate * (instant - segment_start).total_seconds() / 3600
        if cost >= budget:
            instant -= timedelta(hours=budget / rate)
            break
        budget -= cost
        instant = segment_start
    return max(instant, cap).astimezone(_BERLIN)
