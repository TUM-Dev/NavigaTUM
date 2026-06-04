import logging

import polars as pl
from utils import TranslatableStr
from utils import TranslatableStr as _

from processors.df_utils import ensure_column

_logger = logging.getLogger(__name__)


def add_children_properties(lf: pl.LazyFrame) -> pl.LazyFrame:
    """
    Add the "children" and "children_flat" columns to every entry using the "parents" column.

    Direct children come from the last element of each entry's parents list.
    Flat children come from all elements of the parents list.
    """
    # Direct children: last element of parents list is the immediate parent
    direct = lf.select(
        pl.col("id"),
        pl.col("parents").list.last().alias("parent_id"),
    ).filter(pl.col("parent_id").is_not_null())

    children_agg = direct.group_by("parent_id").agg(
        pl.col("id").alias("children"),
    )

    # Flat children: explode all parents
    exploded = (
        lf.select("id", "parents")
        .explode("parents")
        .rename({"parents": "parent_id"})
        .filter(pl.col("parent_id").is_not_null())
    )

    children_flat_agg = exploded.group_by("parent_id").agg(
        pl.col("id").alias("children_flat"),
    )

    # Drop pre-existing children/children_flat columns (they'll be recomputed)
    cols_to_drop = [c for c in lf.collect_schema().names() if c in ("children", "children_flat")]
    if cols_to_drop:
        lf = lf.drop(cols_to_drop)

    # Join back onto main LazyFrame
    lf = lf.join(children_agg, left_on="id", right_on="parent_id", how="left")
    return lf.join(children_flat_agg, left_on="id", right_on="parent_id", how="left")


def add_stats(df: pl.DataFrame) -> pl.DataFrame:
    """
    Calculate structural statistics for each entry (number of rooms, buildings, etc).

    This requires the children_flat column.
    """
    # Log warnings for entries that should have children but don't
    missing_children = df.filter(
        pl.col("type").is_in(["root", "site", "campus", "area"]) & pl.col("children_flat").is_null()
    )
    for row in missing_children.iter_rows(named=True):
        _logger.warning(f"'{row['id']}' ({row['type']}) has no children")

    # Only process entries that have children_flat
    has_children = df.filter(pl.col("children_flat").is_not_null()).select("id", "children_flat")
    if has_children.height == 0:
        return df

    exploded = has_children.explode("children_flat").rename({"children_flat": "child_id", "id": "parent_entry_id"})

    # Join to get child type, usage, and immediate parent
    child_info = df.select(
        pl.col("id"),
        pl.col("type").alias("child_type"),
        pl.col("usage_din_277").alias("child_usage_din_277"),
        pl.col("parents").list.last().alias("child_parent_id"),
    )
    exploded = exploded.join(child_info, left_on="child_id", right_on="id", how="left")

    # Look up the type of each child's immediate parent (for building counting)
    parent_type_lookup = df.select(
        pl.col("id").alias("pp_id"),
        pl.col("type").alias("pp_type"),
    )
    exploded = exploded.join(parent_type_lookup, left_on="child_parent_id", right_on="pp_id", how="left")

    # Count rooms, regular rooms, and buildings in a single group_by
    is_room = pl.col("child_type") == "room"
    is_regular_room = is_room & ~pl.col("child_usage_din_277").fill_null("").str.starts_with("VF")
    is_counted_building = (pl.col("child_type") == "joined_building") | (
        (pl.col("child_type") == "building") & (pl.col("pp_type") != "joined_building")
    )

    counts = (
        exploded.group_by("parent_entry_id")
        .agg(
            pl.col("child_id").filter(is_room).count().alias("_n_rooms"),
            pl.col("child_id").filter(is_regular_room).count().alias("_n_rooms_reg"),
            pl.col("child_id").filter(is_counted_building).count().alias("_n_buildings"),
        )
        .rename({"parent_entry_id": "id"})
    )

    df = df.join(counts, on="id", how="left")

    # Ensure stats columns exist before coalescing
    for col in ["props_stats_n_rooms", "props_stats_n_rooms_reg", "props_stats_n_buildings"]:
        df = ensure_column(df, col, pl.Int64())

    # Apply stats only for applicable types
    applicable_rooms = ["root", "site", "campus", "area", "joined_building", "building"]
    applicable_buildings = ["root", "site", "campus", "area"]

    df = df.with_columns(
        pl.when(pl.col("type").is_in(applicable_rooms) & pl.col("children_flat").is_not_null())
        .then(pl.col("_n_rooms").fill_null(0))
        .otherwise(pl.col("props_stats_n_rooms"))
        .alias("props_stats_n_rooms"),
        pl.when(pl.col("type").is_in(applicable_rooms) & pl.col("children_flat").is_not_null())
        .then(pl.col("_n_rooms_reg").fill_null(0))
        .otherwise(pl.col("props_stats_n_rooms_reg"))
        .alias("props_stats_n_rooms_reg"),
        pl.when(pl.col("type").is_in(applicable_buildings) & pl.col("children_flat").is_not_null())
        .then(pl.col("_n_buildings").fill_null(0))
        .otherwise(pl.col("props_stats_n_buildings"))
        .alias("props_stats_n_buildings"),
    )

    # Log warnings for entries with 0 rooms
    zero_rooms = df.filter(
        pl.col("type").is_in(applicable_rooms)
        & pl.col("children_flat").is_not_null()
        & (pl.col("props_stats_n_rooms") == 0)
    )
    for row in zero_rooms.iter_rows(named=True):
        _logger.warning(f"'{row['id']}' ({row['type']}) has no rooms")

    return df.drop(["_n_rooms", "_n_rooms_reg", "_n_buildings"])


def infer_addresses(df: pl.DataFrame) -> pl.DataFrame:
    """Infer addresses from children. If all children share the same address, assign it to the parent."""
    # Entries that need address inference: no address and has children_flat
    needs_address = df.filter(pl.col("props_address_street").is_null() & pl.col("children_flat").is_not_null()).select(
        "id", "children_flat"
    )

    if needs_address.height == 0:
        return df

    # Explode and join to get child addresses
    exploded = needs_address.explode("children_flat").rename({"children_flat": "child_id"})

    child_addresses = df.select(
        pl.col("id").alias("addr_id"),
        pl.col("props_address_street").alias("child_street"),
        pl.col("props_address_plz_place").alias("child_plz"),
    )
    exploded = exploded.join(child_addresses, left_on="child_id", right_on="addr_id", how="left")

    # Filter to only children that have both street and plz_place
    exploded = exploded.filter(pl.col("child_street").is_not_null() & pl.col("child_plz").is_not_null())

    # Group by parent, check if all children have the same address
    uniform = (
        exploded.group_by("id")
        .agg(
            pl.col("child_street").n_unique().alias("n_streets"),
            pl.col("child_plz").n_unique().alias("n_plz"),
            pl.col("child_street").first().alias("inferred_street"),
            pl.col("child_plz").first().alias("inferred_plz"),
        )
        .filter((pl.col("n_streets") == 1) & (pl.col("n_plz") == 1))
        .select("id", "inferred_street", "inferred_plz")
    )

    if uniform.height == 0:
        return df

    # Join back and fill in missing addresses
    df = df.join(uniform, on="id", how="left")
    return df.with_columns(
        pl.coalesce(pl.col("props_address_street"), pl.col("inferred_street")).alias("props_address_street"),
        pl.coalesce(pl.col("props_address_plz_place"), pl.col("inferred_plz")).alias("props_address_plz_place"),
        pl.when(pl.col("props_address_source").is_null() & pl.col("inferred_street").is_not_null())
        .then(pl.lit("inferred"))
        .otherwise(pl.col("props_address_source"))
        .alias("props_address_source"),
    ).drop(["inferred_street", "inferred_plz"])


TYPE_COMMON_NAME_BY_TYPE: dict[str, str | TranslatableStr] = {
    "root": _("Standortübersicht"),
    "site": _("Standort"),
    "campus": "Campus",
    "area": _("Gebiet / Gruppe von Gebäuden"),
    "joined_building": _("Gebäudekomplex"),
    "building": _("Gebäude"),
    "room": _("Raum"),
    "virtual_room": _("Raum/Gebäudeteil"),
    "poi": "POI",
}


def infer_type_common_name(lf: pl.LazyFrame) -> pl.LazyFrame:
    """Infer the type_common_name property for each entry via the type property."""
    # Build lookup dicts for de and en
    de_map = {}
    en_map = {}
    for t, name in TYPE_COMMON_NAME_BY_TYPE.items():
        if isinstance(name, str):
            de_map[t] = name
            en_map[t] = name
        else:
            de_map[t] = name["de"]
            en_map[t] = name["en"]

    # Look up each entry's immediate parent's type
    parent_type = lf.select(
        pl.col("id").alias("_p_id"),
        pl.col("type").alias("_parent_type"),
    )

    lf = lf.with_columns(
        pl.col("parents").list.last().alias("_last_parent"),
    )
    lf = lf.join(parent_type, left_on="_last_parent", right_on="_p_id", how="left")

    gebaeudeteil = _("Gebäudeteil")

    lf = lf.with_columns(
        # column: type_common_name_de
        pl.when((pl.col("type") == "building") & (pl.col("_parent_type") == "joined_building"))
        .then(pl.lit(gebaeudeteil["de"]))
        .when(pl.col("type").is_in(["room", "virtual_room", "poi"]) & pl.col("usage_name_de").is_not_null())
        .then(pl.col("usage_name_de"))
        .otherwise(pl.col("type").replace_strict(de_map, default=None))
        .alias("type_common_name_de"),
        # column: type_common_name_en
        pl.when((pl.col("type") == "building") & (pl.col("_parent_type") == "joined_building"))
        .then(pl.lit(gebaeudeteil["en"]))
        .when(pl.col("type").is_in(["room", "virtual_room", "poi"]) & pl.col("usage_name_en").is_not_null())
        .then(pl.col("usage_name_en"))
        .otherwise(pl.col("type").replace_strict(en_map, default=None))
        .alias("type_common_name_en"),
    )

    # column type_common_name uses the de value as default
    lf = lf.with_columns(
        pl.col("type_common_name_de").alias("type_common_name"),
    )

    return lf.drop(["_last_parent", "_parent_type"])
