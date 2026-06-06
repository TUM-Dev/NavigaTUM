import logging
from collections.abc import Callable
from pathlib import Path

import polars as pl

from external.scraping_utils import CACHE_PATH
from external.scrapers.iris import IrisRoom, fetch_iris_rooms

# Persisted across builds so a transient AStA outage falls back to the last-known coverage set.
IRIS_COVERAGE_CACHE = CACHE_PATH / "iris_coverage.csv"


def derive_coverage_building_ids(
    iris_rooms: list[IrisRoom],
    navigatum_arch_names: set[str],
) -> set[str]:
    """
    Derive the NavigaTUM building ids that have at least one Iris-matched learning room.

    Each Iris room's `raum_nr_architekt` (the raw `<arch_name>@<building_id>` form) is joined
    against the set of NavigaTUM `arch_name`s. For every matched room, the `@`-suffix is the
    NavigaTUM building id.

    `gebaeude_code` matches NavigaTUM building ids 1:1, so it cross-checks the alias join: any
    building Iris lists but where no room could be alias-matched is logged as a coverage gap
    (it indicates NavigaTUM is missing those rooms' aliases).
    """
    coverage: set[str] = set()
    for room in iris_rooms:
        if room.raum_nr_architekt not in navigatum_arch_names:
            continue
        _, _, building_id = room.raum_nr_architekt.rpartition("@")
        coverage.add(building_id)

    iris_building_ids = {room.gebaeude_code for room in iris_rooms}
    if unmatched := iris_building_ids - coverage:
        logging.warning(
            "Iris lists %d building(s) with no alias-matched room (missing NavigaTUM aliases?): %s",
            len(unmatched),
            sorted(unmatched),
        )
    return coverage


def _load_cached_coverage(cache_path: Path) -> set[str]:
    """Load the previously-persisted coverage set, or an empty set if none exists yet."""
    try:
        # Building ids like "0101" must stay strings, so leading zeros survive the round-trip.
        df = pl.read_csv(cache_path, schema_overrides={"building_id": pl.String})
        return set(df.get_column("building_id").to_list())
    except (OSError, pl.exceptions.PolarsError):
        return set()


def _save_cached_coverage(building_ids: set[str], cache_path: Path) -> None:
    """Persist the coverage set so the next build can fall back to it if Iris is unreachable."""
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    df = pl.DataFrame({"building_id": sorted(building_ids)}, schema={"building_id": pl.String})
    df.write_csv(cache_path)


def add_iris_coverage(
    df: pl.DataFrame,
    *,
    fetch: Callable[[], list[IrisRoom] | None] = fetch_iris_rooms,
    cache_path: Path = IRIS_COVERAGE_CACHE,
) -> pl.DataFrame:
    """
    Add the `has_iris_coverage` signal to every entry.

    Fetches the Iris roster once, derives the covered building ids via the alias join, and
    persists them. If Iris is unreachable, falls back to the previously-known set (empty on the
    first build) so a transient AStA outage cannot break the build. Only building/area entries
    can match (their `id` equals the building id), so rooms naturally get `False`.
    """
    arch_names = set(df.get_column("arch_name").drop_nulls().to_list())
    rooms = fetch()
    if rooms is None:
        building_ids = _load_cached_coverage(cache_path)
        logging.warning(
            "Falling back to previously-known Iris coverage (%d building(s))",
            len(building_ids),
        )
    else:
        building_ids = derive_coverage_building_ids(rooms, arch_names)
        _save_cached_coverage(building_ids, cache_path)
        logging.info("Iris learning-room coverage derived for %d building(s)", len(building_ids))

    return df.with_columns(pl.col("id").is_in(list(building_ids)).alias("has_iris_coverage"))
