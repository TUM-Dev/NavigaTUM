from datetime import date, datetime
from zoneinfo import ZoneInfo

import opening_hours
import pytest

from processors.semester_block_expander import Semester, contains_macro, expand_semester_blocks

# Library hours are wall-clock local time; evaluate state in the library's own zone.
_BERLIN = ZoneInfo("Europe/Berlin")

# The summer semester from the issue example: lectures 22 Apr - 02 Aug, calendar 01 Apr - 30 Sep.
_SS2025 = Semester("2025S", date(2025, 4, 1), date(2025, 4, 22), date(2025, 8, 2), date(2025, 9, 30))
# A following winter semester to exercise multi-semester and year-crossing ranges.
_WS2025 = Semester("2025W", date(2025, 10, 1), date(2025, 10, 13), date(2026, 2, 6), date(2026, 3, 31))
# A semester whose lectures end on a Wednesday, so the lecture/break split lands mid-week.
_MIDWEEK = Semester("2026S", date(2026, 4, 1), date(2026, 4, 13), date(2026, 8, 5), date(2026, 9, 30))


def test_plain_osm_is_returned_verbatim() -> None:
    """A schedule with no macros is passed through untouched - not reformatted."""
    assert expand_semester_blocks("Mo-Fr 08:00-20:00; PH off", [_SS2025]) == "Mo-Fr 08:00-20:00; PH off"


def test_expands_issue_example_lecture_then_break() -> None:
    """`lecture:`/`break:` expand into the lecture range plus the run-up and tail break ranges."""
    out = expand_semester_blocks("lecture: Mo-Fr 08:00-22:00; break: Mo-Fr 09:00-18:00", [_SS2025])

    assert out == (
        "2025 Apr 22-2025 Aug 02 Mo-Fr 08:00-22:00; "
        "2025 Apr 01-2025 Apr 21 Mo-Fr 09:00-18:00; "
        "2025 Aug 03-2025 Sep 30 Mo-Fr 09:00-18:00"
    )
    assert opening_hours.validate(out)


def test_empty_lecture_block_contributes_no_rule() -> None:
    """A bare `lecture:` states nothing; only the `break:` ranges are emitted."""
    out = expand_semester_blocks("lecture: ; break: Mo-Fr 09:00-18:00", [_SS2025])

    assert out == "2025 Apr 01-2025 Apr 21 Mo-Fr 09:00-18:00; 2025 Aug 03-2025 Sep 30 Mo-Fr 09:00-18:00"
    assert opening_hours.validate(out)


def test_multiple_semesters_each_get_a_dated_range() -> None:
    """A `lecture:` block expands once per semester, ordered by lecture start."""
    out = expand_semester_blocks("lecture: Mo-Fr 08:00-20:00", [_WS2025, _SS2025])

    assert out == "2025 Apr 22-2025 Aug 02 Mo-Fr 08:00-20:00; 2025 Oct 13-2026 Feb 06 Mo-Fr 08:00-20:00"
    assert opening_hours.validate(out)


def test_semester_boundary_mid_week_splits_lecture_and_break() -> None:
    """Lectures ending on a Wednesday split that week: Mo-We lecture hours, Th-Fr break hours."""
    out = expand_semester_blocks("lecture: Mo-Fr 08:00-22:00; break: Mo-Fr 09:00-18:00", [_MIDWEEK])

    assert out == (
        "2026 Apr 13-2026 Aug 05 Mo-Fr 08:00-22:00; "
        "2026 Apr 01-2026 Apr 12 Mo-Fr 09:00-18:00; "
        "2026 Aug 06-2026 Sep 30 Mo-Fr 09:00-18:00"
    )

    hours = opening_hours.OpeningHours(out)
    assert date(2026, 8, 5).weekday() == 2, "boundary day must be a Wednesday for this test to mean anything"
    # Wednesday is still lecture time (open until 22:00); Thursday has flipped to break (closed by 18:00).
    assert hours.is_open(datetime(2026, 8, 5, 20, 0, tzinfo=_BERLIN))
    assert not hours.is_open(datetime(2026, 8, 6, 20, 0, tzinfo=_BERLIN))
    assert hours.is_open(datetime(2026, 8, 6, 10, 0, tzinfo=_BERLIN))


def test_break_omits_empty_run_up_when_lectures_start_on_day_one() -> None:
    """When the lecture period starts on the calendar's first day, no run-up break range is emitted."""
    flush = Semester("flush", date(2025, 4, 1), date(2025, 4, 1), date(2025, 8, 2), date(2025, 9, 30))

    out = expand_semester_blocks("break: Mo-Fr 09:00-18:00", [flush])

    assert out == "2025 Aug 03-2025 Sep 30 Mo-Fr 09:00-18:00"


def test_plain_block_interleaved_with_macros_is_kept_unconditional() -> None:
    """A non-macro rule among macro blocks stays in place and applies unconditionally."""
    out = expand_semester_blocks("PH off; lecture: Mo-Fr 08:00-20:00", [_SS2025])

    assert out == "PH off; 2025 Apr 22-2025 Aug 02 Mo-Fr 08:00-20:00"
    assert opening_hours.validate(out)


def test_macro_with_no_semesters_yields_empty_schedule() -> None:
    """With no semesters there is nothing to expand a macro against."""
    assert expand_semester_blocks("lecture: Mo-Fr 08:00-20:00", []) == ""


@pytest.mark.parametrize(
    ("schedule", "expected"),
    [
        ("Mo-Fr 08:00-20:00", False),
        ('Mo-Fr 08:00-20:00 "Coffee break"', False),
        ("lecture: Mo-Fr 08:00-20:00", True),
        ("Mo-Fr 08:00-20:00; break: 12:00-13:00", True),
    ],
)
def test_contains_macro(schedule: str, expected: bool) -> None:
    """`contains_macro` recognises `lecture:`/`break:` prefixes but not incidental words."""
    assert contains_macro(schedule) is expected
