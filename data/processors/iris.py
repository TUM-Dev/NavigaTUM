import logging

import polars as pl
from external.loaders.iris import load_iris_rooms
from external.schemas.iris import IrisRoomsSchema

_logger = logging.getLogger(__name__)


def derive_coverage_building_ids(iris_rooms: pl.DataFrame, navigatum_arch_names: set[str]) -> set[str]:
    """
    Derive the NavigaTUM building ids that have at least one Iris-matched learning room.

    Each Iris room's `raum_nr_architekt` (the raw `<arch_name>@<building_id>` form) is joined
    against the set of NavigaTUM `arch_name`s; the `@`-suffix of a matched room is its building id.

    `gebaeude_code` matches NavigaTUM building ids 1:1, so any building Iris lists but where no
    room could be alias-matched is logged: it flags a NavigaTUM alias gap, not an Iris problem.
    """
    matched = iris_rooms.filter(pl.col("raum_nr_architekt").is_in(list(navigatum_arch_names)))
    coverage = set(matched.get_column("raum_nr_architekt").str.split("@").list.last().to_list())

    iris_building_ids = set(iris_rooms.get_column("gebaeude_code").to_list())
    if unmatched := iris_building_ids - coverage:
        _logger.warning(
            "Iris lists %d building(s) with no alias-matched room (missing NavigaTUM aliases?): %s",
            len(unmatched),
            sorted(unmatched),
        )
    return coverage


def add_iris_coverage(df: pl.DataFrame, *, rooms: pl.DataFrame | None = None) -> pl.DataFrame:
    """
    Add the non-nullable `iris_coverage_building_ids` list to every entry.

    A container (area, campus, the MI `joined_building`, …) inherits the coverage of its descendant
    buildings via `children_flat`, so the aggregate page can offer the learning-room view too.

    On the first build there is no scrape yet, so every list comes out empty.
    """
    if rooms is None:
        rooms = _load_stored_rooms()
    arch_names = set(df.get_column("arch_name").drop_nulls().to_list())
    building_ids = derive_coverage_building_ids(rooms, arch_names)
    _logger.info("Iris learning-room coverage derived for %d building(s)", len(building_ids))

    # children_flat is null for leaves, so coalesce before concatenating.
    candidates = pl.concat_list(
        pl.col("id"),
        pl.col("children_flat").fill_null(pl.lit([], dtype=pl.List(pl.Utf8))),
    )
    coverage = (
        candidates.list.eval(pl.element().filter(pl.element().is_in(list(building_ids))))
        .list.unique()
        .list.sort()
        .alias("iris_coverage_building_ids")
    )
    return df.with_columns(coverage)


def _load_stored_rooms() -> pl.DataFrame:
    try:
        return load_iris_rooms()
    except FileNotFoundError:
        _logger.warning("No stored Iris roster yet; iris_coverage_building_ids will be empty this build")
        return pl.DataFrame(schema=IrisRoomsSchema.to_polars_schema())
