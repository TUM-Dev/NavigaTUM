from typing import Any

from processors import merge
from utils import TranslatableStr as _


def merge_poi(data: dict[str, dict[str, Any]]) -> None:
    """Merge POIs from `sources/21_pois.yaml` into the data"""
    poi_data = merge.load_yaml("sources/21_pois.yaml")

    for _id, poi in poi_data.items():
        if _id in data:
            raise ValueError(f"The id '{_id}' is already used, cannot use it for a new POI")

        if poi["parent"] not in data:
            raise ValueError(f"Parent '{poi['parent']}' of POI '{_id}' not found")

        poi["type"] = "poi"
        poi["id"] = _id

        # make sure that name and usage is internationalized
        poi["usage"]["name"] = _(poi["usage"]["name"])
        poi["name"] = _(poi["name"])
        links = poi.get("description", {}).get("links", [])
        for link in links:
            link["text"] = _(link["text"])

        parent_id = poi.pop("parent")
        parent = data[parent_id]
        poi["parents"] = parent["parents"] + [parent["id"]]

        poi.setdefault("sources", {"base": [{"name": "NavigaTUM"}]})

    return merge.recursively_merge(data, poi_data)
