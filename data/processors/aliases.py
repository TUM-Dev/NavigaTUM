import polars as pl


def building_short_name_lookup(meta: pl.DataFrame) -> pl.DataFrame:
    """
    For each entry, the code-like short_name that prefixes its rooms' arch names.

    Walks ``[self, immediate_parent, ..., root]`` nearest-first and stops at the first
    geographic ancestor or the first ancestor carrying any short_name. A non-code-like
    short_name on a building-like ancestor is a deliberate "no usable prefix here"
    signal and prevents borrowing from further up.
    """
    chains = (
        meta.lazy()
        .select(
            pl.col("id"),
            pl.concat_list(pl.col("id"), pl.col("parents").list.reverse()).alias("ancestor_id"),
        )
        .explode("ancestor_id")
        .with_columns(pl.int_range(pl.len()).over("id").alias("depth"))
    )

    # `root` is in every chain but not always a meta row; the join leaves its type null,
    # which the is_in below folds into the geographic branch.
    ancestor_attrs = meta.lazy().select(
        pl.col("id").alias("ancestor_id"),
        pl.col("type").alias("ancestor_type"),
        pl.col("short_name").alias("ancestor_short_name"),
    )
    chains = chains.join(ancestor_attrs, on="ancestor_id", how="left").sort("id", "depth")

    is_building_like = pl.col("ancestor_type").is_in(["building", "joined_building"]).fill_null(value=False)
    is_stop = ((~is_building_like) | pl.col("ancestor_short_name").is_not_null()).cast(pl.Int32)

    chains = chains.with_columns(is_stop.cum_sum().over("id").alias("stops_seen"))
    first_stops = chains.filter((pl.col("stops_seen") == 1) & (is_stop == 1))

    return (
        first_stops.filter(
            is_building_like & pl.col("ancestor_short_name").str.contains(r"^[A-Za-z0-9]+$"),
        )
        .select(
            pl.col("id"),
            pl.col("ancestor_short_name").alias("building_short_name"),
        )
        .collect()
    )


def add_aliases(lf: pl.LazyFrame, short_name_lookup: pl.DataFrame) -> pl.LazyFrame:
    """
    Add ``arch_name`` and ``aliases`` columns.

    TUMonline supplies arch_names as ``"<number>@<building_id>"`` (e.g. ``0001@5510``),
    which never matches room-code queries like ``MW0001``. When ``short_name_lookup``
    has a code-like short_name for the building, we emit a friendly
    ``"<short_name><number>"`` alias next to the raw form so existing links keep
    working while room-code searches resolve correctly.
    """
    extracted_arch = (
        pl.when(pl.col("tumonline_data_json").is_not_null())
        .then(pl.col("tumonline_data_json").str.json_path_match("$.arch_name"))
        .otherwise(pl.lit(None))
    )
    lf = lf.with_columns(
        pl.when(pl.col("type") == "building")
        .then(pl.lit("@") + pl.col("id"))
        .otherwise(extracted_arch)
        .alias("arch_name"),
    )
    # json_path_match returns "" for missing keys; downstream uses null tests.
    lf = lf.with_columns(
        pl.when(pl.col("arch_name") == "").then(pl.lit(None)).otherwise(pl.col("arch_name")).alias("arch_name"),
    )

    # split_exact's right half is null when "@" is missing, so malformed arch_names miss the join.
    arch_parts = pl.col("arch_name").str.split_exact("@", 1)
    lf = lf.with_columns(
        arch_parts.struct.field("field_0").alias("_arch_number"),
        arch_parts.struct.field("field_1").alias("_arch_building_id"),
    )

    lookup = short_name_lookup.lazy().rename({"id": "_arch_building_id"})
    lf = lf.join(lookup, on="_arch_building_id", how="left")

    # Buildings have arch_name "@<id>" (number == ""), so they decline a self-alias.
    friendly_alias = (
        pl.when(pl.col("building_short_name").is_not_null() & (pl.col("_arch_number") != ""))
        .then(pl.col("building_short_name") + pl.col("_arch_number"))
        .otherwise(pl.lit(None))
    )

    aliases_list = pl.concat_list(pl.col("arch_name"), friendly_alias).list.drop_nulls()
    lf = lf.with_columns(aliases_list.alias("aliases"))

    return lf.drop("_arch_number", "_arch_building_id", "building_short_name")
