import logging
from datetime import UTC, date, datetime
from pathlib import Path

import dataframely as dy
import polars as pl
from external.freshness import warn_if_stale
from external.loaders.studierendenwerk import load_studierendenwerk
from external.schemas.opening_hours import OpeningHoursSchema
from external.schemas.studierendenwerk import StudierendenwerkSchema

_logger = logging.getLogger(__name__)

SOURCES_PATH = Path(__file__).parent.parent / "sources"
CANTEEN_MAPPING_CSV = SOURCES_PATH / "mensa_canteens.csv"
_MAPPING_SCHEMA = {"canteen_id": pl.Utf8(), "id": pl.Utf8()}


def _load_mapping() -> pl.DataFrame:
    """Load the hand-authored canteen-id -> NavigaTUM entry-id mapping."""
    return pl.read_csv(CANTEEN_MAPPING_CSV, schema=_MAPPING_SCHEMA)


def _load_stored_canteens() -> pl.DataFrame:
    try:
        return load_studierendenwerk()
    except FileNotFoundError:
        _logger.warning("No stored canteen feed yet; no mensa opening hours will be attached this build")
        return pl.DataFrame(schema=StudierendenwerkSchema.to_polars_schema())


def mensa_opening_hours(
    *,
    canteens: pl.DataFrame | None = None,
    mapping: pl.DataFrame | None = None,
    today: date | None = None,
) -> dy.DataFrame[OpeningHoursSchema]:
    """
    Map scraped canteen hours onto NavigaTUM entry ids as `OpeningHoursSchema` records.

    Canteens are matched to entries via the hand-authored `mensa_canteens.csv`. A mapped
    canteen absent from the feed is logged (mapping drift or an upstream rename); an
    unmapped canteen is ignored. Emits a build-time staleness warning per feed snapshot.
    `canteens`/`mapping`/`today` are injectable for tests.
    """
    canteens = _load_stored_canteens() if canteens is None else canteens
    mapping = _load_mapping() if mapping is None else mapping
    today = datetime.now(tz=UTC).date() if today is None else today

    joined = mapping.join(canteens, on="canteen_id", how="left")
    for row in joined.filter(pl.col("opening_hours").is_null()).iter_rows(named=True):
        _logger.warning("mensa mapping references canteen %r absent from the eat-api feed", row["canteen_id"])
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


def stamp_canteen_ids(df: pl.DataFrame, *, mapping: pl.DataFrame | None = None) -> pl.DataFrame:
    """
    Stamp the eat-api canteen slug onto each mapped entry as a `mensa_canteen_id` column.

    The webclient gates its menu card on this slug and fetches the live menu client-side from
    `GET /api/mensa/{slug}`, so no menu payload is baked into the build. A mapping referencing an
    entry id absent from `df` fails the build (mapping drift). `mapping` is injectable for tests.
    """
    mapping = _load_mapping() if mapping is None else mapping
    unknown = set(mapping["id"]) - set(df["id"])
    if unknown:
        raise ValueError(f"mensa_canteens.csv references unknown entry id(s): {sorted(unknown)}")
    stamp = mapping.select(pl.col("id"), pl.col("canteen_id").alias("mensa_canteen_id"))
    return df.join(stamp, on="id", how="left")
