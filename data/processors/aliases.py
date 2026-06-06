import re

import polars as pl

# Types whose short_name we are willing to borrow when walking up from a building.
# We deliberately stop at geographic groupings (area/campus/site): a campus short_name
# like "Garching" is not a meaningful room-code prefix.
_BUILDING_LIKE_TYPES = {"building", "joined_building"}

# Only short_names that read as a room-code prefix become friendly arch-name aliases.
# This excludes descriptive multi-word short_names ("Mathe/Info (MI)", "Business Campus 1")
# that would otherwise produce nonsensical aliases like "Mathe/Info (MI)0001".
_CODE_LIKE_SHORT_NAME = re.compile(r"^[A-Za-z0-9]+$")


def building_short_name_lookup(df: pl.DataFrame) -> dict[str, str]:
    """
    Map every entry id to the code-like short_name that prefixes its rooms' arch names.

    An entry uses its own short_name when it has one; otherwise it borrows the nearest
    ``building``/``joined_building`` ancestor's short_name (e.g. building ``5510`` has no
    short_name of its own but sits under the ``mw`` joined_building, so it resolves to ``MW``).
    The walk stops at the first non-building-like ancestor so geographic short_names never leak in.

    Only short_names matching :data:`_CODE_LIKE_SHORT_NAME` are returned; others cannot form a
    sensible ``<short_name><number>`` alias and are dropped here so the caller stays simple.
    """
    by_id = {row["id"]: row for row in df.select(["id", "type", "short_name", "parents"]).to_dicts()}

    lookup: dict[str, str] = {}
    for entry_id, row in by_id.items():
        # nearest-first: the entry itself, then its parents from immediate up to root
        for ancestor_id in [entry_id, *reversed(row["parents"] or [])]:
            ancestor = by_id.get(ancestor_id)
            if ancestor is None or ancestor["type"] not in _BUILDING_LIKE_TYPES:
                break
            if short_name := ancestor["short_name"]:
                if _CODE_LIKE_SHORT_NAME.match(short_name):
                    lookup[entry_id] = short_name
                break
    return lookup


def add_aliases(lf: pl.LazyFrame, short_name_lookup: dict[str, str]) -> pl.LazyFrame:
    """
    Add arch_name and aliases_json columns.

    For buildings: arch_name = "@" + id
    For others with tumonline_data_json containing arch_name: extract it via JSON path.

    TUMonline supplies arch_names as ``<number>@<building_id>`` (e.g. ``0001@5510``). When the
    building resolves to a code-like short_name via :func:`building_short_name_lookup`, we also
    emit a friendly ``<short_name><number>`` alias (e.g. ``MW0001``) so that room-code searches
    resolve to the correct entry. The raw upstream form is always kept so existing links survive.

    Returns a LazyFrame with arch_name and aliases_json columns added.
    """
    # Extract arch_name from tumonline_data_json for non-buildings
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

    # Null out empty arch_names (json_path_match returns "" for empty values)
    lf = lf.with_columns(
        pl.when(pl.col("arch_name") == "").then(pl.lit(None)).otherwise(pl.col("arch_name")).alias("arch_name"),
    )

    # Derive the friendly "<short_name><number>" form from "<number>@<building_id>" arch_names.
    arch_parts = pl.col("arch_name").str.split("@")
    number = arch_parts.list.get(0, null_on_oob=True)
    building_id = arch_parts.list.get(1, null_on_oob=True)
    building_short_name = building_id.replace_strict(short_name_lookup, default=None)
    friendly_alias = (
        pl.when(building_short_name.is_not_null() & (number != ""))
        .then(building_short_name + number)
        .otherwise(pl.lit(None))
    )
    lf = lf.with_columns(friendly_alias.alias("_friendly_alias"))

    # aliases_json: JSON array string of [arch_name, friendly_alias?], else empty array.
    lf = lf.with_columns(
        pl.when(pl.col("arch_name").is_null())
        .then(pl.lit("[]"))
        .when(pl.col("_friendly_alias").is_null())
        .then(pl.lit('["') + pl.col("arch_name") + pl.lit('"]'))
        .otherwise(pl.lit('["') + pl.col("arch_name") + pl.lit('","') + pl.col("_friendly_alias") + pl.lit('"]'))
        .alias("aliases_json"),
    )

    return lf.drop("_friendly_alias")
