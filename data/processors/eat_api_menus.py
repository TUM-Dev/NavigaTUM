import logging
from datetime import UTC, date, datetime
from pathlib import Path

import orjson
import polars as pl
from external.freshness import warn_if_stale
from external.loaders.eat_api_menus import load_eat_api_menus
from external.schemas.eat_api_menus import EatApiMenuSchema

_logger = logging.getLogger(__name__)

SOURCES_PATH = Path(__file__).parent.parent / "sources"
CANTEEN_MAPPING_CSV = SOURCES_PATH / "mensa_canteens.csv"
_MAPPING_SCHEMA = {"canteen_id": pl.Utf8(), "id": pl.Utf8()}


def _load_mapping() -> pl.DataFrame:
    """Load the hand-authored canteen-id -> NavigaTUM entry-id mapping."""
    return pl.read_csv(CANTEEN_MAPPING_CSV, schema=_MAPPING_SCHEMA)


def _load_stored_menus() -> pl.DataFrame:
    try:
        return load_eat_api_menus()
    except FileNotFoundError:
        _logger.warning("No stored eat-api menu feed yet; no mensa menus will be attached this build")
        return pl.DataFrame(schema=EatApiMenuSchema.to_polars_schema())


def _build_payload(rows: list[dict]) -> dict:
    """
    Re-nest a sorted slice of dish rows for one canteen into a `MenuResponse`-shaped payload.

    `rows` must already be sorted by `(date, position)` so days and dishes appear in serving
    order. `source_url`/`last_update` are taken from the latest row, mirroring opening-hours
    behaviour when several scraper snapshots land in the same build.
    """
    days: list[dict] = []
    current_day: dict | None = None
    for row in rows:
        if current_day is None or current_day["date"] != row["date"]:
            current_day = {"date": row["date"], "dishes": []}
            days.append(current_day)
        dish: dict = {
            "name": row["name"],
            "prices": orjson.loads(row["prices_json"]),
            "labels": orjson.loads(row["labels_json"]),
        }
        if row["dish_type"]:
            dish["dish_type"] = row["dish_type"]
        current_day["dishes"].append(dish)
    return {
        "source_url": rows[-1]["source_url"],
        "last_update": rows[-1]["last_update"],
        "days": days,
    }


def merge_mensa_menus(
    df: pl.DataFrame,
    *,
    menus: pl.DataFrame | None = None,
    mapping: pl.DataFrame | None = None,
    today: date | None = None,
) -> pl.DataFrame:
    """
    Attach scraped eat-api menus to their entries as a `mensa_menus_json` payload.

    Canteens are matched to entries via the hand-authored `mensa_canteens.csv`. A mapped
    canteen absent from the feed is logged (mapping drift or an upstream rename); an
    unmapped canteen is ignored. Emits a build-time staleness warning per feed snapshot.
    `menus`/`mapping`/`today` are injectable for tests.
    """
    menus = _load_stored_menus() if menus is None else menus
    mapping = _load_mapping() if mapping is None else mapping
    today = today or datetime.now(UTC).date()

    joined = mapping.join(menus, on="canteen_id", how="left")
    missing_ids = joined.filter(pl.col("name").is_null()).select("canteen_id").unique().to_series().to_list()
    for canteen_id in missing_ids:
        _logger.warning("mensa mapping references canteen %r absent from the eat-api menu feed", canteen_id)
    joined = joined.filter(pl.col("name").is_not_null())
    if joined.is_empty():
        return df

    unknown = set(joined["id"]) - set(df["id"])
    if unknown:
        raise ValueError(f"eat-api menus reference unknown entry id(s): {sorted(unknown)}")

    for row in joined.select("source_url", "last_update").unique().iter_rows(named=True):
        warn_if_stale(date.fromisoformat(row["last_update"]), today=today, source=row["source_url"])

    joined = joined.sort(["id", "date", "position"])
    payloads: list[dict[str, str]] = []
    for entry_id, group in joined.group_by("id", maintain_order=True):
        rows = group.iter_rows(named=True)
        payload = _build_payload(list(rows))
        payloads.append({"id": entry_id[0], "mensa_menus_json": orjson.dumps(payload).decode()})

    encoded = pl.DataFrame(payloads, schema={"id": pl.Utf8(), "mensa_menus_json": pl.Utf8()})
    return df.join(encoded, on="id", how="left")
