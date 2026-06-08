from datetime import date, datetime
from zoneinfo import ZoneInfo

from processors.opening_hours_state import evaluate_state
from processors.public_holiday_expander import expand_public_holidays
from processors.semester_block_expander import Semester, expand_semester_blocks

# Library hours are wall-clock local time; evaluate state in the library's own zone.
_BERLIN = ZoneInfo("Europe/Berlin")
# A plain weekday schedule reused across the cardinal cases.
_WEEKDAYS = "Mo-Fr 08:00-22:00"


def _at(year: int, month: int, day: int, hour: int, minute: int = 0) -> datetime:
    return datetime(year, month, day, hour, minute, tzinfo=_BERLIN)


def test_monday_morning_is_open_until_evening() -> None:
    """A Monday mid-morning reads open, with `until` the same day's closing time."""
    state = evaluate_state(_WEEKDAYS, _at(2026, 6, 8, 9, 0))  # 2026-06-08 is a Monday.

    assert state.state == "open"
    assert state.until == _at(2026, 6, 8, 22, 0)
    assert state.next_change is None


def test_friday_closing_minute_flips_from_open_to_closed() -> None:
    """One minute before close is still open; the closing minute itself is closed."""
    just_before = evaluate_state(_WEEKDAYS, _at(2026, 6, 5, 21, 59))  # 2026-06-05 is a Friday.
    assert just_before.state == "open"
    assert just_before.until == _at(2026, 6, 5, 22, 0)

    at_close = evaluate_state(_WEEKDAYS, _at(2026, 6, 5, 22, 0))
    assert at_close.state == "closed"
    # The weekend is closed, so the next opening is the following Monday morning.
    assert at_close.next_change == _at(2026, 6, 8, 8, 0)


def test_sunday_is_closed_until_monday_morning() -> None:
    """Sunday reads closed with the next opening on Monday."""
    state = evaluate_state(_WEEKDAYS, _at(2026, 6, 7, 15, 0))  # 2026-06-07 is a Sunday.

    assert state.state == "closed"
    assert state.until is None
    assert state.next_change == _at(2026, 6, 8, 8, 0)


def test_bavarian_holiday_is_closed_despite_weekday_hours() -> None:
    """A `PH off` schedule, expanded against a Bavarian holiday, is closed on that weekday."""
    epiphany = date(2026, 1, 6)  # Heilige Drei Könige - a Tuesday, Bavaria-only holiday.
    schedule = expand_public_holidays(f"{_WEEKDAYS}; PH off", [epiphany])

    state = evaluate_state(schedule, _at(2026, 1, 6, 10, 0))

    assert state.state == "closed"
    # Closed all day; reopens the next working day, Wednesday.
    assert state.next_change == _at(2026, 1, 7, 8, 0)


def test_mid_break_uses_break_hours_not_lecture_hours() -> None:
    """Inside the semester break the shorter break hours apply, not the lecture-period hours."""
    # Lectures start 13 Apr 2026, so the calendar run-up (01-12 Apr) is break time.
    semester = Semester("2026S", date(2026, 4, 1), date(2026, 4, 13), date(2026, 8, 5), date(2026, 9, 30))
    schedule = expand_semester_blocks(f"lecture: {_WEEKDAYS}; break: Mo-Fr 09:00-18:00", [semester])
    monday_in_break = (2026, 4, 6)  # 2026-04-06 is a Monday in the run-up break.

    before_break_open = evaluate_state(schedule, _at(*monday_in_break, 8, 30))
    assert before_break_open.state == "closed"  # 08:30 is before the 09:00 break opening.
    assert before_break_open.next_change == _at(*monday_in_break, 9, 0)

    during = evaluate_state(schedule, _at(*monday_in_break, 9, 30))
    assert during.state == "open"
    assert during.until == _at(*monday_in_break, 18, 0)  # break closes at 18:00, not the lecture 22:00.


def test_always_open_has_no_closing_time() -> None:
    """A `24/7` schedule is open with no upcoming change."""
    state = evaluate_state("24/7", _at(2026, 6, 8, 3, 0))

    assert state.state == "open"
    assert state.until is None
    assert state.next_change is None


def test_schedule_with_only_past_intervals_is_closed_forever() -> None:
    """A schedule whose every interval is in the past is closed with no next change."""
    state = evaluate_state("2020 Jan 01 09:00-17:00", _at(2026, 6, 8, 10, 0))

    assert state.state == "closed"
    assert state.next_change is None
