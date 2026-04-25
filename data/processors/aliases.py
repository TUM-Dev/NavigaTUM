import polars as pl


def add_aliases(lf: pl.LazyFrame) -> pl.LazyFrame:
    """
    Add arch_name and aliases_json columns.

    For buildings: arch_name = "@" + id
    For others with tumonline_data_json containing arch_name: extract it via JSON path.

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

    # aliases_json: JSON array string with arch_name if present, else empty array
    lf = lf.with_columns(
        pl.when(pl.col("arch_name").is_not_null())
        .then(pl.lit('["') + pl.col("arch_name") + pl.lit('"]'))
        .otherwise(pl.lit("[]"))
        .alias("aliases_json"),
    )

    return lf
