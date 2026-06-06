import polars as pl

# Walks stop here so geographic short_names ("Garching", "Innenstadt") never leak into room aliases.
_BUILDING_LIKE_TYPES = ["building", "joined_building"]

# Descriptive short_names ("Mathe/Info (MI)") would yield nonsensical "Mathe/Info (MI)0001" aliases.
_CODE_LIKE_SHORT_NAME = r"^[A-Za-z0-9]+$"


def building_short_name_lookup(meta: pl.DataFrame) -> pl.DataFrame:
    """
    Map every entry id to the code-like short_name that prefixes its rooms' arch names.

    Walks each entry's ancestor chain ``[self, immediate_parent, ..., root]`` nearest-first
    and stops at the first ancestor that is either geographic OR carries any short_name.
    The entry receives a row only when that stop ancestor is building-like and its
    short_name matches :data:`_CODE_LIKE_SHORT_NAME`; a non-code-like short_name on a
    building-like ancestor is treated as a deliberate "no usable prefix here" signal
    and prevents borrowing from further up the chain.

    Returns a DataFrame with columns ``id`` and ``building_short_name``.
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

    # `root` appears in every chain but is not always materialised as a meta row; the
    # left join produces nulls for it, which the is_in below folds into the geographic branch.
    ancestor_attrs = meta.lazy().select(
        pl.col("id").alias("ancestor_id"),
        pl.col("type").alias("ancestor_type"),
        pl.col("short_name").alias("ancestor_short_name"),
    )
    chains = chains.join(ancestor_attrs, on="ancestor_id", how="left").sort("id", "depth")

    is_building_like = pl.col("ancestor_type").is_in(_BUILDING_LIKE_TYPES).fill_null(value=False)
    is_stop = ((~is_building_like) | pl.col("ancestor_short_name").is_not_null()).cast(pl.Int32)

    chains = chains.with_columns(is_stop.cum_sum().over("id").alias("stops_seen"))
    first_stops = chains.filter((pl.col("stops_seen") == 1) & (is_stop == 1))

    return (
        first_stops.filter(
            is_building_like & pl.col("ancestor_short_name").str.contains(_CODE_LIKE_SHORT_NAME),
        )
        .select(
            pl.col("id"),
            pl.col("ancestor_short_name").alias("building_short_name"),
        )
        .collect()
    )


def add_aliases(lf: pl.LazyFrame, short_name_lookup: pl.DataFrame) -> pl.LazyFrame:
    """
    Add ``arch_name`` and ``aliases`` columns to ``lf``.

    Buildings synthesise an ``"@<id>"`` arch_name; every other entry inherits one from
    ``tumonline_data_json``. When the arch_name has the canonical
    ``"<number>@<building_id>"`` shape and ``short_name_lookup`` has a code-like
    short_name for that building, a friendly ``"<short_name><number>"`` alias is emitted
    alongside the raw form so room-code searches (e.g. ``MW0001``) resolve correctly
    without breaking existing links.

    ``aliases`` is a ``List[Utf8]`` of the resulting alias strings; empty when no
    arch_name is known.
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
    # json_path_match returns "" for missing keys; the rest of the pipeline uses null tests.
    lf = lf.with_columns(
        pl.when(pl.col("arch_name") == "").then(pl.lit(None)).otherwise(pl.col("arch_name")).alias("arch_name"),
    )

    # split_exact yields null for the right half when arch_name lacks "@", so malformed
    # arch_names naturally miss the join below and derive no friendly alias.
    arch_parts = pl.col("arch_name").str.split_exact("@", 1)
    lf = lf.with_columns(
        arch_parts.struct.field("field_0").alias("_arch_number"),
        arch_parts.struct.field("field_1").alias("_arch_building_id"),
    )

    lookup = short_name_lookup.lazy().rename({"id": "_arch_building_id"})
    lf = lf.join(lookup, on="_arch_building_id", how="left")

    # Buildings carry arch_name = "@<id>", i.e. number == "", so they decline to derive
    # a friendly alias against themselves.
    friendly_alias = (
        pl.when(pl.col("building_short_name").is_not_null() & (pl.col("_arch_number") != ""))
        .then(pl.col("building_short_name") + pl.col("_arch_number"))
        .otherwise(pl.lit(None))
    )

    # concat_list keeps the friendly form null when absent; drop_nulls strips it so the
    # resulting list contains only the actually-present alias forms. An entry with no
    # arch_name at all returns an empty list rather than [null].
    aliases_list = pl.concat_list(pl.col("arch_name"), friendly_alias).list.drop_nulls()
    lf = lf.with_columns(aliases_list.alias("aliases"))

    return lf.drop("_arch_number", "_arch_building_id", "building_short_name")
