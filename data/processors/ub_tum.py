import logging
from datetime import UTC, date, datetime
from pathlib import Path

import dataframely as dy
import polars as pl
from external.freshness import warn_if_stale
from external.loaders.ub_tum import load_ub_tum
from external.schemas.opening_hours import OpeningHoursSchema
from external.schemas.ub_tum import UbTumSchema

_logger = logging.getLogger(__name__)

SOURCES_PATH = Path(__file__).parent.parent / "sources"
LIBRARY_MAPPING_CSV = SOURCES_PATH / "ub_tum_libraries.csv"
_MAPPING_SCHEMA = {"branch_id": pl.Utf8(), "id": pl.Utf8()}


def _load_mapping() -> pl.DataFrame:
    return pl.read_csv(LIBRARY_MAPPING_CSV, schema=_MAPPING_SCHEMA)


def _load_stored_branches() -> pl.DataFrame:
    try:
        return load_ub_tum()
    except FileNotFoundError:
        _logger.warning("No stored UB-TUM scrape yet, no library opening hours will be attached this build")
        return pl.DataFrame(schema=UbTumSchema.to_polars_schema())


def ub_tum_opening_hours(
    *,
    branches: pl.DataFrame | None = None,
    mapping: pl.DataFrame | None = None,
    today: date | None = None,
) -> dy.DataFrame[OpeningHoursSchema]:
    """Map scraped UB-TUM branch hours onto NavigaTUM entry ids via `ub_tum_libraries.csv`."""
    branches = _load_stored_branches() if branches is None else branches
    mapping = _load_mapping() if mapping is None else mapping
    today = datetime.now(tz=UTC).date() if today is None else today

    joined = mapping.join(branches, on="branch_id", how="left")
    for row in joined.filter(pl.col("opening_hours").is_null()).iter_rows(named=True):
        _logger.warning("UB-TUM mapping references branch %r absent from the scrape", row["branch_id"])
    joined = joined.filter(pl.col("opening_hours").is_not_null())

    for row in joined.select("source_url", "last_update").unique().iter_rows(named=True):
        warn_if_stale(date.fromisoformat(row["last_update"]), today=today, source=row["source_url"])

    records = joined.select(
        pl.col("id"),
        pl.col("opening_hours"),
        pl.col("source_url"),
        pl.col("last_update"),
        pl.lit(None, dtype=pl.Utf8()).alias("valid_from"),
        pl.lit(None, dtype=pl.Utf8()).alias("valid_until"),
        pl.lit(None, dtype=pl.Utf8()).alias("service"),
    )
    return OpeningHoursSchema.validate(records)
