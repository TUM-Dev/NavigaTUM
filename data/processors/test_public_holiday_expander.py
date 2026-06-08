from datetime import date, datetime
from zoneinfo import ZoneInfo

import opening_hours
import pytest

from processors.public_holiday_expander import (
    bavarian_holiday_dates,
    contains_ph,
    expand_public_holidays,
)

# Library hours are wall-clock local time; evaluate state in the library's own zone.
_BERLIN = ZoneInfo("Europe/Berlin")
# Epiphany (Bavaria-only) and Fronleichnam (Bavaria-only) in 2026, both ordinary weekdays.
_EPIPHANY = date(2026, 1, 6)  # a Tuesday.
_FRONLEICHNAM = date(2026, 6, 4)  # a Thursday.


def test_plain_osm_without_ph_is_returned_verbatim() -> None:
    """A schedule with no `PH` selector is passed through untouched."""
    assert expand_public_holidays("Mo-Fr 08:00-22:00", [_EPIPHANY]) == "Mo-Fr 08:00-22:00"


def test_ph_off_becomes_one_dated_off_rule_per_holiday() -> None:
    """`PH off` expands to a dated `off` override per holiday, appended after the regular rules."""
    out = expand_public_holidays("Mo-Fr 08:00-22:00; PH off", [_EPIPHANY, _FRONLEICHNAM])

    assert out == "Mo-Fr 08:00-22:00; 2026 Jan 06 off; 2026 Jun 04 off"
    assert opening_hours.validate(out)
    hours = opening_hours.OpeningHours(out)
    # The holiday overrides the weekday schedule; the surrounding Tuesdays stay open.
    assert not hours.is_open(datetime(2026, 1, 6, 10, 0, tzinfo=_BERLIN))
    assert hours.is_open(datetime(2026, 1, 13, 10, 0, tzinfo=_BERLIN))


def test_ph_closed_keyword_is_carried_through() -> None:
    """`closed` (an `off` synonym) is preserved in the dated rule."""
    out = expand_public_holidays("Mo-Fr 08:00-22:00; PH closed", [_EPIPHANY])

    assert out == "Mo-Fr 08:00-22:00; 2026 Jan 06 closed"
    assert opening_hours.validate(out)


def test_ph_with_times_expands_to_dated_time_ranges() -> None:
    """A `PH <times>` rule (open only on holidays) expands to dated time ranges."""
    out = expand_public_holidays("Mo-Su 00:00-24:00; PH 10:00-14:00", [_EPIPHANY])

    assert out == "Mo-Su 00:00-24:00; 2026 Jan 06 10:00-14:00"
    assert opening_hours.validate(out)


def test_ph_rule_overrides_regardless_of_its_position() -> None:
    """A leading `PH off` still wins: its dated overrides are appended after the weekday rule."""
    out = expand_public_holidays("PH off; Mo-Fr 08:00-22:00", [_EPIPHANY])

    assert out == "Mo-Fr 08:00-22:00; 2026 Jan 06 off"
    assert not opening_hours.OpeningHours(out).is_open(datetime(2026, 1, 6, 10, 0, tzinfo=_BERLIN))


def test_no_holidays_drops_the_ph_rule() -> None:
    """With no holidays in range the `PH` rule has no effect and is dropped."""
    assert expand_public_holidays("Mo-Fr 08:00-22:00; PH off", []) == "Mo-Fr 08:00-22:00"


def test_duplicate_holidays_are_collapsed_and_ordered() -> None:
    """Holiday dates are de-duplicated and emitted in calendar order."""
    out = expand_public_holidays("Mo-Fr 08:00-22:00; PH off", [_FRONLEICHNAM, _EPIPHANY, _EPIPHANY])

    assert out == "Mo-Fr 08:00-22:00; 2026 Jan 06 off; 2026 Jun 04 off"


def test_combined_ph_selector_is_rejected() -> None:
    """A `PH` combined with weekdays is not supported and must raise rather than mis-expand."""
    with pytest.raises(ValueError, match="combined PH selector"):
        expand_public_holidays("Mo-Fr,PH off", [_EPIPHANY])


@pytest.mark.parametrize(
    ("schedule", "expected"),
    [
        ("Mo-Fr 08:00-22:00", False),
        ('Mo-Fr 08:00-22:00 "Pharmacy"', False),
        ("PH off", True),
        ("Mo-Fr 08:00-22:00; PH off", True),
    ],
)
def test_contains_ph(schedule: str, expected: bool) -> None:
    """`contains_ph` recognises a `PH` selector but not incidental letters."""
    assert contains_ph(schedule) is expected


def test_bavarian_holiday_dates_include_bavaria_only_days() -> None:
    """The Bavarian calendar carries regional holidays national calendars omit, sorted ascending."""
    dates = bavarian_holiday_dates([2026])

    assert _EPIPHANY in dates  # Bavaria-only; absent from a national DE calendar.
    assert date(2026, 11, 1) in dates  # Allerheiligen, also Bavaria-only.
    assert dates == sorted(dates)
