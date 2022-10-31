import json
from pathlib import Path
from typing import Any, Union

OUTPUT_DIR = Path(__file__).parent.parent / "output"


def unlocalise(value: Union[str, list[Any], dict[str, Any]]) -> Any:
    """Recursively unlocalise a dictionary"""
    if isinstance(value, (bool, float, int, str)) or value is None:
        return value
    if isinstance(value, list):
        return [unlocalise(v) for v in value]
    if isinstance(value, dict):
        # We consider each dict that has only the keys "de" and/or "en" as translated string
        if set(value.keys()) | {"de", "en"} == {"de", "en"}:
            # Since we only unlocalise dicts with either en and/or de or {}, the default to {} is fine
            return value.get("de", value.get("en", {}))

        return {k: unlocalise(v) for k, v in value.items()}
    raise ValueError(f"Unhandled type {type(value)}")


def export_for_search(data, path):
    """export a subset of the data for the /search api"""
    export = []
    for _id, _data in data.items():
        # Currently, the "root" entry is excluded from search
        if _id == "root":
            continue

        building_parents_index = len(_data["parents"])
        if _data["type"] in {"room", "virtual_room"}:
            for i, parent in enumerate(_data["parents"]):
                if data[parent]["type"] in {"building", "joined_building"}:
                    building_parents_index = i
                    break

        # The 'campus name' is the campus of site of this building or room
        campus_name = None
        if _data["type"] not in {"root", "campus", "site"}:
            for parent in _data["parents"]:
                if data[parent]["type"] in {"campus", "site"}:
                    campus = data[parent]
                    campus_name = campus.get("short_name", campus["name"])
                    # intentionally no break, because sites might be below a campus

        export.append(
            {
                # MeiliSearch requires an id without "."
                # also this puts more emphasis on the order (because "." counts as more distance)
                "ms_id": _id.replace(".", "-"),
                "id": _id,  # not searchable
                "name": _data["name"],
                "arch_name": _data.get("tumonline_data", {}).get("arch_name", None),
                "type": _data["type"],
                "type_common_name": _data["type_common_name"],
                "facet": {
                    "site": "site",
                    "campus": "site",
                    "area": "site",
                    "joined_building": "building",
                    "building": "building",
                    "room": "room",
                    "virtual_room": "room",
                }.get(_data["type"], None),
                # Parents always exclude root
                # "parent_names": _data["parents"][1:], [data[p]["name"] for p in _data["parents"][1:]],
                # For rooms, the (joined_)building parents are extra to put more emphasis on them.
                # Also their name is included
                "parent_building_names": [
                    data[p]["short_name"] for p in _data["parents"][building_parents_index:] if "short_name" in data[p]
                ]
                + [data[p]["name"] for p in _data["parents"][building_parents_index:]],
                # For all other parents, only the ids and their keywords (TODO) are searchable
                "parent_keywords": _data["parents"][1:],
                "campus": campus_name,
                "address": _data.get("tumonline_data", {}).get("address", None),
                "usage": _data.get("usage", {}).get("name", None),
                "rank": int(_data["ranking_factors"]["rank_combined"]),
            },
        )

    # the data contains translations, currently we dont allow these in the search api
    export = unlocalise(export)

    with open(path, "w", encoding="utf-8") as file:
        json.dump(export, file)


def export_for_api(data, path):
    """Add some more information about parents to the data and export for the /get/:id api"""

    export_data = {}
    for _id, entry in data.items():
        if entry["type"] != "root":
            entry.setdefault("maps", {})["default"] = "interactive"

        # For the transition from the old roomfinder we export an arch_name similar
        # to the one used by the old roomfinder. For rooms it is like "<room name>@<building id>"
        # and for buildings like "@<building id>". For everything else this field is None.
        if entry["type"] == "building":
            arch_name = f"@{entry['id']}"
        else:
            arch_name = entry.get("tumonline_data", {}).get("arch_name", None)
        export_data[_id] = {
            "parent_names": [data[p]["name"] for p in entry["parents"]],
            "arch_name": arch_name,
            **entry,
        }
        if "children" in export_data[_id]:
            del export_data[_id]["children"]
            del export_data[_id]["children_flat"]
        if "tumonline_data" in export_data[_id]:
            del export_data[_id]["tumonline_data"]
        if "roomfinder_data" in export_data[_id]:
            del export_data[_id]["roomfinder_data"]
        if "props" in export_data[_id]:
            prop_keys_to_keep = {"computed", "links", "comment", "calendar_url"}
            to_delete = [e for e in export_data[_id]["props"].keys() if e not in prop_keys_to_keep]
            for k in to_delete:
                del export_data[_id]["props"][k]

    with open(path, "w", encoding="utf-8") as file:
        json.dump(export_data, file)
