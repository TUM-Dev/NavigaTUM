from processors import merge


def merge_poi(data):
    """Merge POIs from `sources/21_pois.yaml` into the data"""
    poi_data = merge.load_yaml("sources/21_pois.yaml")

    for _id, poi in poi_data.items():
        if _id in data:
            raise ValueError(f"The id '{_id}' is already used, cannot use it for a new POI")

        if poi["parent"] not in data:
            raise ValueError(f"Parent '{poi['parent']}' of POI '{_id}' not found")

        poi["type"] = "poi"

        parent = data[poi["parent"]]
        poi["parents"] = parent["parents"] + [parent["id"]]
        del poi["parent"]

        poi.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "NavigaTUM",
            },
        )

    return merge.recursively_merge(data, poi_data)
