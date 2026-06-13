from datetime import datetime, timedelta
from zoneinfo import ZoneInfo

from processors.events_appears_at import compute_appears_at

_BERLIN = ZoneInfo("Europe/Berlin")


def _window_hours(starts_at: datetime) -> float:
    """Lead-in length: real hours between the computed appears_at and starts_at."""
    return (starts_at - compute_appears_at(starts_at, starts_at)).total_seconds() / 3600


def test_eight_hour_floor_reaches_a_short_weekday_talk() -> None:
    """A 40-minute Wednesday-daytime talk still earns the 8h budget, drained at the 1.0/0.5 rates."""
    starts_at = datetime(2026, 6, 17, 14, 0, tzinfo=_BERLIN)  # Wednesday.
    ends_at = starts_at + timedelta(minutes=40)

    # 4h daytime (10:00-14:00) spends 4 of the 8h budget; the remaining 4 drains at
    # the 0.5 night rate, reaching back 8 real hours to 02:00.
    assert compute_appears_at(starts_at, ends_at) == datetime(2026, 6, 17, 2, 0, tzinfo=_BERLIN)


def test_same_talk_on_saturday_rides_longer() -> None:
    """The identical talk on a Saturday sees less traffic, so its window stretches well past the weekday one."""
    weekday = datetime(2026, 6, 17, 14, 0, tzinfo=_BERLIN)  # Wednesday.
    saturday = datetime(2026, 6, 20, 14, 0, tzinfo=_BERLIN)  # Saturday.
    forty_min = timedelta(minutes=40)

    assert _window_hours(saturday) > _window_hours(weekday)
    # Sat 14:00 -> Sat 10:00 (weekend day, 0.5) spends 2; Sat 10:00 -> Sat 00:00
    # (weekend night, 0.25) spends 2.5; Fri 18:00-00:00 (weekday night, 0.5) spends
    # 3; the last 0.5 drains over 1 real hour of Friday daytime (1.0) to Fri 17:30.
    assert compute_appears_at(saturday, saturday + forty_min) == datetime(2026, 6, 19, 17, 30, tzinfo=_BERLIN)


def test_friday_to_saturday_straddle_switches_rate_at_midnight() -> None:
    """Walking back across Saturday 00:00 flips the weekend factor, so the Friday side drains at the weekday rate."""
    # Sat 02:00 start, 40 min: Sat 00:00-02:00 is weekend night (0.25) spending 0.5;
    # the budget then crosses into Friday, where night is the weekday rate (0.5).
    starts_at = datetime(2026, 6, 20, 2, 0, tzinfo=_BERLIN)  # Saturday.
    ends_at = starts_at + timedelta(minutes=40)

    appears_at = compute_appears_at(starts_at, ends_at)
    # 0.25*2h = 0.5 spent on the weekend side; 7.5 budget left drains over Friday's
    # 18:00-00:00 night (0.5 -> 3) then back into 10:00-18:00 daytime (1.0) for the
    # remaining 4.5, reaching Fri 13:30.
    assert appears_at == datetime(2026, 6, 19, 13, 30, tzinfo=_BERLIN)
    assert appears_at.weekday() == 4  # Friday: the straddle crossed midnight.


def test_forty_eight_hour_cap_bites_on_a_multi_day_event() -> None:
    """A multi-day event's budget would reach back days, but the window is capped at 48h before starts_at."""
    starts_at = datetime(2026, 6, 15, 16, 0, tzinfo=_BERLIN)
    ends_at = datetime(2026, 6, 19, 23, 59, tzinfo=_BERLIN)  # ~4.3 days, budget >> 48h.

    assert compute_appears_at(starts_at, ends_at) == starts_at - timedelta(hours=48)


def test_dst_spring_forward_night_counts_only_real_hours() -> None:
    """The spring-forward night is 9 real hours, not 10 wall-clock hours, so the budget drains accordingly."""
    # 2026-03-29 02:00 -> 03:00 local; clocks before the gap run at +01:00.
    starts_at = datetime(2026, 3, 29, 12, 0, tzinfo=_BERLIN)  # Sunday, +02:00.
    ends_at = starts_at + timedelta(minutes=40)

    # Sun 12:00->10:00 weekend day (0.5) spends 1; the 00:00-10:00 weekend night
    # (0.25) costs only 0.25*9=2.25 because that night is 9 real hours; Sat
    # 18:00-00:00 (0.25) spends 1.5; the last 3.25 drains over Sat daytime (0.5)
    # for 6.5 real hours, reaching Sat 11:30 (+01:00).
    assert compute_appears_at(starts_at, ends_at) == datetime(2026, 3, 28, 11, 30, tzinfo=_BERLIN)


def test_result_is_independent_of_input_offset() -> None:
    """The same instant expressed in UTC or in Berlin local yields the same appears_at."""
    berlin = datetime(2026, 6, 17, 14, 0, tzinfo=_BERLIN)
    as_utc = berlin.astimezone(ZoneInfo("UTC"))

    assert compute_appears_at(berlin, berlin) == compute_appears_at(as_utc, as_utc)


def test_naive_datetimes_are_rejected() -> None:
    """Wall-clock classification is meaningless without an offset, so naive inputs raise."""
    naive = datetime(2026, 6, 17, 14, 0)  # noqa: DTZ001 - the missing tzinfo is the point of this test.
    try:
        compute_appears_at(naive, naive)
    except ValueError:
        return
    raise AssertionError("expected ValueError for naive datetime")
