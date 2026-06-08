import logging
from pathlib import Path

import orjson
import polars as pl
from pipeline_types import FlatRow
from utils import TranslatableStr as _

from processors import merge
from processors.df_utils import translatable_to_columns

_logger = logging.getLogger(__name__)

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"


def merge_poi(df: pl.DataFrame) -> pl.DataFrame:
    """Merge POIs from `sources/21_pois.yaml` into the data."""
    pois_path = SOURCES_PATH / "21_pois.yaml"
    poi_data = merge.load_yaml(pois_path)

    existing_ids = set(df["id"].to_list())
    # Build parent lookup: id -> parents list
    parent_lookup = dict(zip(df["id"].to_list(), df["parents"].to_list(), strict=True))

    new_rows: list[FlatRow] = []
    for _id, poi in poi_data.items():
        if _id in existing_ids:
            raise ValueError(f"The id '{_id}' is already used, cannot use it for a new POI")

        parent_id = poi.get("parent")
        if parent_id not in parent_lookup:
            raise ValueError(f"Parent '{parent_id}' of POI '{_id}' not found")

        parents = parent_lookup[parent_id] + [parent_id]

        # Internationalize name and usage
        poi_name = _(poi["name"])
        usage_name = _(poi["usage"]["name"])

        # Internationalize description links
        links = poi.get("description", {}).get("links", [])
        for link in links:
            link["text"] = _(link["text"])

        row: FlatRow = {
            "id": _id,
            "type": "poi",
            "parents": parents,
            "sources_base_json": orjson.dumps([{"name": "NavigaTUM"}]).decode(),
        }

        # Name columns
        row["name"] = poi_name.get("de", poi_name.get("en", "")) if isinstance(poi_name, dict) else str(poi_name)
        row.update(translatable_to_columns("name", poi_name))

        # Usage columns
        row.update(translatable_to_columns("usage_name", usage_name))

        # Coords
        if "coords" in poi:
            row["coords_lat"] = poi["coords"].get("lat")
            row["coords_lon"] = poi["coords"].get("lon")
            if "source" in poi["coords"]:
                row["coords_source"] = poi["coords"]["source"]

        # Description
        if "description" in poi:
            row["description_json"] = orjson.dumps(poi["description"]).decode()

        # Props - links are kept as plain strings, localize_links handles them later
        if "props" in poi:
            if "links" in poi["props"]:
                row["props_links_json"] = orjson.dumps(poi["props"]["links"]).decode()
            if "comment" in poi["props"]:
                c = poi["props"]["comment"]
                if isinstance(c, dict):
                    row["props_comment_de"] = c.get("de", "")
                    row["props_comment_en"] = c.get("en", "")
            if "generic" in poi["props"]:
                row["props_generic_json"] = orjson.dumps(poi["props"]["generic"]).decode()

        # Generators
        if "generators" in poi:
            row["generators_json"] = orjson.dumps(poi["generators"]).decode()

        new_rows.append(row)

        # Register new POI so subsequent POIs can use it as a parent
        existing_ids.add(_id)
        parent_lookup[_id] = parents

    if new_rows:
        new_df = pl.DataFrame(new_rows, infer_schema_length=None)
        df = pl.concat([df, new_df], how="diagonal_relaxed")

    return df


def propagate_poi_floors(df: pl.DataFrame) -> pl.DataFrame:
    """
    Copy the immediate parent's `props_floors_json` onto each POI row.

    POIs don't get floors assigned by `sections.compute_floor_prop` (which only
    targets `[building, joined_building, site, campus]` and their room children
    with a roomcode). Without floors, `FloorControl.setAvailableFloors([])`
    dims every button and the indoor overlay never displays.

    For a room-parented POI this yields a single-floor list (auto-selected by
    `DetailsInteractiveMap.vue`). For a building-parented POI it yields the
    full building floor list (user picks).
    """
    pois = df.filter(pl.col("type") == "poi").select(
        "id",
        pl.col("parents").list.last().alias("parent_id"),
    )
    if pois.height == 0:
        return df

    parent_floors = df.select(
        pl.col("id").alias("parent_id"),
        pl.col("props_floors_json").alias("parent_floors_json"),
    )
    pois_with_parent = pois.join(parent_floors, on="parent_id", how="left")

    for row in pois_with_parent.filter(pl.col("parent_floors_json").is_null()).iter_rows(named=True):
        _logger.warning(f"POI {row['id']}: parent {row['parent_id']} has no floors")

    updates = pois_with_parent.filter(pl.col("parent_floors_json").is_not_null()).select(
        "id",
        pl.col("parent_floors_json").alias("props_floors_json_new"),
    )
    if updates.height == 0:
        return df

    return (
        df.join(updates, on="id", how="left")
        .with_columns(
            pl.coalesce(pl.col("props_floors_json_new"), pl.col("props_floors_json")).alias("props_floors_json"),
        )
        .drop("props_floors_json_new")
    )
