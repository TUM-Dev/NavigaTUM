"""
Build-time staleness check for scraped opening-hours sources.

A source whose feed has not refreshed in months is a signal that an upstream layout
change broke the scraper before users notice stale hours (cf. #1087). `today` is
injected so the check is deterministic under test. Shared so #3106's UB-library
scraper can reuse the same threshold.
"""

import calendar
import logging
from datetime import date

_logger = logging.getLogger(__name__)

DEFAULT_MAX_AGE_MONTHS = 6


def _months_before(reference: date, months: int) -> date:
    """Return the date `months` calendar months before `reference`, clamping the day-of-month."""
    month_index = reference.month - 1 - months
    year = reference.year + month_index // 12
    month = month_index % 12 + 1
    day = min(reference.day, calendar.monthrange(year, month)[1])
    return date(year, month, day)


def is_stale(last_update: date, *, today: date, max_age_months: int = DEFAULT_MAX_AGE_MONTHS) -> bool:
    """Return True when `last_update` is more than `max_age_months` calendar months before `today`."""
    return last_update < _months_before(today, max_age_months)


def warn_if_stale(
    last_update: date,
    *,
    today: date,
    source: str,
    max_age_months: int = DEFAULT_MAX_AGE_MONTHS,
    logger: logging.Logger = _logger,
) -> bool:
    """Log a warning and return `True` when `source` is stale; otherwise return `False`."""
    if is_stale(last_update, today=today, max_age_months=max_age_months):
        logger.warning(
            "opening-hours source %r is stale: last refreshed %s, more than %d months before %s",
            source,
            last_update.isoformat(),
            max_age_months,
            today.isoformat(),
        )
        return True
    return False
