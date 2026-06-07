from pathlib import Path
from typing import Any

import orjson
import polars as pl
import yaml
from utils import TranslatableStr

from processors.df_utils import flatten_entry

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"
COORDINATES_CSV = SOURCES_PATH / "coordinates.csv"
COMMENTS_CSV = SOURCES_PATH / "comments.csv"
NAMES_CSV = SOURCES_PATH / "names.csv"
RANKING_CSV = SOURCES_PATH / "ranking.csv"
USAGES_CSV = SOURCES_PATH / "usages.csv"
LINKS_YAML = SOURCES_PATH / "links.yaml"


def load_yaml(path: Path) -> Any:
    """
    Merge yaml data at path on top of the given data.

    This operates on the data dict directly without creating a copy.
    """

    def add_translatable_str(value: str | list[Any] | dict[str, Any]) -> Any:
        """Recursively change all {de: ..., en:...] to a TranslatableStr"""
        if isinstance(value, bool | float | int | str) or value is None:
            return value
        if isinstance(value, list):
            return [add_translatable_str(v) for v in value]
        if isinstance(value, dict):
            # We consider each dict that has only the keys "de" and "en" as translated string
            if set(value.keys()) == {"de", "en"}:
                return TranslatableStr(value["de"], value["en"])

            return {k: add_translatable_str(v) for k, v in value.items()}
        raise ValueError(f"Unhandled type {type(value)}")

    with path.open(encoding="utf-8") as file:
        yaml_data = yaml.safe_load(file.read())
    yaml_data = add_translatable_str(yaml_data)

    if not isinstance(yaml_data, dict):
        raise TypeError(f"Error: root node expected to be an object in file '{path}'")

    # If the key of a root element is only numeric with 4 digits,
    # we assume it is a building id (which needs to be converted to string)
    for _id, _data in list(yaml_data.items()):
        if isinstance(_id, int) and len(str(_id)) == 4:
            yaml_data[str(_id)] = yaml_data[_id]
            del yaml_data[_id]

    return yaml_data


def add_coordinates(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge coordinates from CSV file on top of the given DataFrame.

    Reads coordinates.csv (id, lat, lon) and left-joins onto df,
    filling coords_lat/coords_lon only where they were previously null.
    """
    coords_df = pl.read_csv(COORDINATES_CSV, schema_overrides={"id": pl.Utf8})

    # Validate that all coordinate ids exist in the main df
    unknown = coords_df.join(df.select("id"), on="id", how="anti")
    if unknown.height > 0:
        raise RuntimeError(f"Coordinates exist for entries which should not exist: {set(unknown['id'].to_list())}")

    coords_df = coords_df.rename({"lat": "coords_lat__csv", "lon": "coords_lon__csv"})
    result = df.join(coords_df, on="id", how="left")

    # CSV coordinates overwrite existing (matches original recursively_merge with overwrite=True)
    coalesce_exprs = [
        pl.coalesce(pl.col("coords_lat__csv"), pl.col("coords_lat")).alias("coords_lat"),
        pl.coalesce(pl.col("coords_lon__csv"), pl.col("coords_lon")).alias("coords_lon"),
    ]

    result = result.with_columns(coalesce_exprs)
    return result.drop(["coords_lat__csv", "coords_lon__csv"])


def _apply_patch_df(df: pl.DataFrame, patch_df: pl.DataFrame) -> pl.DataFrame:
    """
    Left-join patch_df onto df and coalesce so patch values win over originals.

    New rows from patch_df (ids not in df) are also added.
    """
    if patch_df.is_empty():
        return df

    # Align dtypes: cast patch columns to match df where they overlap
    cast_exprs = [
        pl.col(col).cast(df.schema[col])
        for col in patch_df.columns
        if col != "id" and col in df.columns and patch_df.schema[col] != df.schema[col]
    ]
    if cast_exprs:
        patch_df = patch_df.with_columns(cast_exprs)

    common_cols = [c for c in patch_df.columns if c != "id" and c in df.columns]

    # Rename patch columns to avoid collision
    rename_map = {c: f"{c}__patch" for c in common_cols}
    renamed = patch_df.rename(rename_map)

    # Full outer join so new ids from patch are included
    result = df.join(renamed, on="id", how="full", coalesce=True)

    # Coalesce: patch wins over original
    coalesce_exprs = [pl.coalesce(pl.col(f"{col}__patch"), pl.col(col)).alias(col) for col in common_cols]

    if coalesce_exprs:
        result = result.with_columns(coalesce_exprs)

    # Drop the __patch columns
    drop_cols = [f"{c}__patch" for c in common_cols]
    return result.drop(drop_cols)


def _yaml_to_patch_df(yaml_data: dict[str, dict[str, Any]]) -> pl.DataFrame:
    """Convert a YAML dict of {id: entry_dict} to a flat DataFrame using flatten_entry."""
    rows = []
    for entry_id, entry in yaml_data.items():
        row = flatten_entry(entry_id, entry)
        # Remove keys with None/empty values to avoid overwriting existing data
        row = {k: v for k, v in row.items() if v is not None and v != []}
        rows.append(row)
    if not rows:
        return pl.DataFrame({"id": []})
    return pl.DataFrame(rows, infer_schema_length=None)


def patch_areas(df: pl.DataFrame) -> pl.DataFrame:
    """Merge areas from the yaml file at path on top of the given DataFrame."""
    areas_extended = SOURCES_PATH / "01_areas-extended.yaml"
    yaml_data = load_yaml(areas_extended)
    patch_df = _yaml_to_patch_df(yaml_data)
    return _apply_patch_df(df, patch_df)


def add_comments(df: pl.DataFrame) -> pl.DataFrame:
    """Merge comments from comments.csv (id, de, en) into props_comment_de/en."""
    if not COMMENTS_CSV.exists():
        raise FileNotFoundError(f"Required source file not found: {COMMENTS_CSV}")
    comments_df = pl.read_csv(COMMENTS_CSV, schema_overrides={"id": pl.Utf8})
    comments_df = comments_df.rename({"de": "props_comment_de", "en": "props_comment_en"})
    return _apply_patch_df(df, comments_df)


def add_names(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge name/short_name overrides from names.csv (id, name, short_name, arch_name).

    Creates new entries if ids don't exist yet. For room names, prepends the
    room id if not already in the name: "id (name)".
    """
    if not NAMES_CSV.exists():
        raise FileNotFoundError(f"Required source file not found: {NAMES_CSV}")
    names_df = pl.read_csv(NAMES_CSV, schema_overrides={"id": pl.Utf8})

    # Prepend room id to name if not already present (matches patch_rooms behavior)
    if "name" in names_df.columns:
        names_df = names_df.with_columns(
            pl.when(
                pl.col("name").is_not_null()
                & pl.col("name").str.len_chars().gt(0)
                & ~pl.col("name").str.contains(pl.col("id"))
            )
            .then(pl.col("id") + pl.lit(" (") + pl.col("name") + pl.lit(")"))
            .otherwise(pl.col("name"))
            .alias("name"),
        )

    # Build a patch DataFrame with the right column names
    rows = []
    for row in names_df.iter_rows(named=True):
        flat: dict[str, Any] = {"id": row["id"]}
        name = row.get("name")
        if name:
            flat["name"] = name
            flat["name_de"] = name
            flat["name_en"] = name
        short = row.get("short_name")
        if short:
            flat["short_name"] = short
            flat["short_name_de"] = short
            flat["short_name_en"] = short
        arch = row.get("arch_name")
        if arch:
            flat["arch_name"] = arch
        rows.append(flat)

    if not rows:
        return df
    patch_df = pl.DataFrame(rows, infer_schema_length=None)
    # Remove None-valued columns to avoid overwriting with nulls
    patch_df = patch_df.select([c for c in patch_df.columns if patch_df[c].null_count() < patch_df.height])
    return _apply_patch_df(df, patch_df)


def add_ranking(df: pl.DataFrame) -> pl.DataFrame:
    """Merge custom ranking from ranking.csv (id, rank_custom)."""
    if not RANKING_CSV.exists():
        raise FileNotFoundError(f"Required source file not found: {RANKING_CSV}")
    ranking_df = pl.read_csv(RANKING_CSV, schema_overrides={"id": pl.Utf8, "rank_custom": pl.Int64})
    ranking_df = ranking_df.rename({"rank_custom": "ranking_rank_custom"})
    return _apply_patch_df(df, ranking_df)


def add_usages(df: pl.DataFrame) -> pl.DataFrame:
    """Merge usage overrides from usages.csv (id, name_de, name_en, din_277, din_277_desc)."""
    if not USAGES_CSV.exists():
        raise FileNotFoundError(f"Required source file not found: {USAGES_CSV}")
    usages_df = pl.read_csv(USAGES_CSV, schema_overrides={"id": pl.Utf8})
    # Rename columns to match the flat schema: name_de -> usage_name_de, etc.
    rename_map = {}
    for col in usages_df.columns:
        if col != "id":
            rename_map[col] = f"usage_{col}" if not col.startswith("usage_") else col
    usages_df = usages_df.rename(rename_map)
    return _apply_patch_df(df, usages_df)


def add_links(df: pl.DataFrame) -> pl.DataFrame:
    """Merge links from links.yaml (id -> list of {text, url}) into props_links_json."""
    if not LINKS_YAML.exists():
        raise FileNotFoundError(f"Required source file not found: {LINKS_YAML}")
    with LINKS_YAML.open(encoding="utf-8") as f:
        links_data = yaml.safe_load(f)
    if not links_data:
        return df

    rows = []
    for entry_id, links in links_data.items():
        rows.append({"id": str(entry_id), "props_links_json__yaml": orjson.dumps(links).decode()})

    links_df = pl.DataFrame(rows, infer_schema_length=None)
    result = df.join(links_df, on="id", how="left")
    return result.with_columns(
        pl.coalesce(pl.col("props_links_json__yaml"), pl.col("props_links_json")).alias("props_links_json"),
    ).drop("props_links_json__yaml")
