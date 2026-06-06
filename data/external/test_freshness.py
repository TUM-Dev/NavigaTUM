import logging
from datetime import date

import pytest

from external.freshness import is_stale, warn_if_stale


def test_recent_update_is_not_stale() -> None:
    """An update inside the window is fresh."""
    assert not is_stale(date(2026, 1, 15), today=date(2026, 6, 7))


def test_old_update_is_stale() -> None:
    """An update older than the window is stale."""
    assert is_stale(date(2025, 11, 1), today=date(2026, 6, 7))


def test_staleness_boundary_is_exclusive() -> None:
    """Exactly `max_age_months` before today is the last fresh day; one day earlier is stale."""
    assert not is_stale(date(2025, 12, 7), today=date(2026, 6, 7))
    assert is_stale(date(2025, 12, 6), today=date(2026, 6, 7))


def test_month_arithmetic_clamps_day_of_month() -> None:
    """Six months before Aug 31 lands on the last day of February, not an invalid date."""
    assert not is_stale(date(2026, 2, 28), today=date(2026, 8, 31))
    assert is_stale(date(2026, 2, 27), today=date(2026, 8, 31))


def test_warn_if_stale_logs_and_reports_true(caplog: pytest.LogCaptureFixture) -> None:
    """A stale source logs a warning and returns True."""
    with caplog.at_level(logging.WARNING):
        assert warn_if_stale(date(2025, 1, 1), today=date(2026, 6, 7), source="https://x.tld")
    assert "stale" in caplog.text
    assert "https://x.tld" in caplog.text


def test_warn_if_stale_is_quiet_when_fresh(caplog: pytest.LogCaptureFixture) -> None:
    """A fresh source emits no warning and returns False."""
    with caplog.at_level(logging.WARNING):
        assert not warn_if_stale(date(2026, 6, 1), today=date(2026, 6, 7), source="https://x.tld")
    assert caplog.text == ""
