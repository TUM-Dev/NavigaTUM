import logging

import polars as pl

from external.models.iris import IrisRoom


def derive_coverage_building_ids(
    iris_rooms: list[IrisRoom],
    navigatum_arch_names: set[str],
) -> set[str]:
    """
    Derive the NavigaTUM building ids that have at least one Iris-matched learning room.

    Each Iris room's `raum_nr_architekt` (the raw `<arch_name>@<building_id>` form) is joined
    against the set of NavigaTUM `arch_name`s; the `@`-suffix of a matched room is its building id.

    `gebaeude_code` matches NavigaTUM building ids 1:1, so any building Iris lists but where no
    room could be alias-matched is logged: it flags a NavigaTUM alias gap, not an Iris problem.
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


def add_iris_coverage(df: pl.DataFrame, *, rooms: list[IrisRoom] | None = None) -> pl.DataFrame:
    """
    Add the non-nullable `has_iris_coverage` flag to every entry.

    The roster is read from the committed scrape; on the first build (before any scrape) it is
    empty, so nothing is marked. Only building/area entries can match (their `id` equals the
    building id), so rooms get `False`.
    """
    if rooms is None:
        rooms = _load_stored_rooms()
    arch_names = set(df.get_column("arch_name").drop_nulls().to_list())
    building_ids = derive_coverage_building_ids(rooms, arch_names)
    logging.info("Iris learning-room coverage derived for %d building(s)", len(building_ids))
    return df.with_columns(pl.col("id").is_in(list(building_ids)).alias("has_iris_coverage"))


def _load_stored_rooms() -> list[IrisRoom]:
    try:
        return IrisRoom.load_all()
    except FileNotFoundError:
        logging.warning("No stored Iris roster yet; has_iris_coverage will be empty this build")
        return []
