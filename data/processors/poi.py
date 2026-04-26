from pathlib import Path
from typing import Any

import orjson
import polars as pl
from utils import TranslatableStr as _

from processors import merge
from processors.df_utils import translatable_to_columns

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"


def merge_poi(df: pl.DataFrame) -> pl.DataFrame:
    """Merge POIs from `sources/21_pois.yaml` into the data."""
    pois_path = SOURCES_PATH / "21_pois.yaml"
    poi_data = merge.load_yaml(pois_path)

    existing_ids = set(df["id"].to_list())
    # Build parent lookup: id -> parents list
    parent_lookup = dict(zip(df["id"].to_list(), df["parents"].to_list(), strict=True))

    new_rows: list[dict[str, Any]] = []
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

        row: dict[str, Any] = {
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
