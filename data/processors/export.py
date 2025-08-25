import json
from pathlib import Path
from typing import Any, Union

from external.models.common import PydanticConfiguration

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


def export_for_search(data: dict, path: str) -> None:
    """export a subset of the data for the /search api"""
    export = []
    for _id, entry in data.items():
        # Currently, the "root" entry is excluded from search
        if _id == "root":
            continue

        building_parents_index = len(entry["parents"])
        if entry["type"] in {"room", "virtual_room"}:
            for i, parent in enumerate(entry["parents"]):
                if data[parent]["type"] in {"building", "joined_building"}:
                    building_parents_index = i
                    break

        # The 'campus name' is the campus of site of this building or room
        campus_name = None
        if entry["type"] not in {"root", "campus", "site"}:
            for parent in entry["parents"]:
                if data[parent]["type"] in {"campus", "site"}:
                    campus = data[parent]
                    campus_name = campus.get("short_name", campus["name"])
                    # intentionally no break, because sites might be below a campus

        geo = {}
        if coords := entry.get("coords"):
            geo["_geo"] = {"lat": coords["lat"], "lon": coords["lon"]}
        export.append(
            {
                # MeiliSearch requires an id without "."
                # also this puts more emphasis on the order (because "." counts as more distance)
                "ms_id": _id.replace(".", "-"),
                "id": _id,  # not searchable
                "name": entry["name"],
                "arch_name": extract_arch_name(entry),
                "type": entry["type"],
                "type_common_name": entry["type_common_name"],
                "facet": {
                    "site": "site",
                    "campus": "site",
                    "area": "site",
                    "joined_building": "building",
                    "building": "building",
                    "room": "room",
                    "virtual_room": "room",
                }.get(entry["type"]),
                "parent_building_names": extract_parent_building_names(data, entry["parents"], building_parents_index),
                # For all other parents, only the ids and their keywords (TODO) are searchable
                "parent_keywords": entry["parents"][1:],
                "campus": campus_name,
                "address": entry.get("tumonline_data", {}).get("address", None),
                "usage": entry.get("usage", {}).get("name", None),
                "rank": int(entry["ranking_factors"]["rank_combined"]),
                **geo
            },
        )

    # the data contains translations, currently we don't allow these in the search api
    export = unlocalise(export)

    with open(path, "w", encoding="utf-8") as file:
        json.dump(export, file)


def extract_parent_building_names(data: dict, parents: list, building_parents_index: int) -> list:
    """Extract the parents building names from the data"""
    # For rooms, the (joined_)building parents are extra to put more emphasis on them.
    short_names = [data[p]["short_name"] for p in parents[building_parents_index:] if "short_name" in data[p]]
    long_names = [data[p]["name"] for p in parents[building_parents_index:]]
    return short_names + long_names


def extract_arch_name(entry: dict) -> str | None:
    """Extract the arch name from the entry"""
    if entry["type"] == "building":
        return f"@{entry['id']}"
    return entry.get("tumonline_data", {}).get("arch_name", None)


def export_for_api(data: dict, path: str) -> None:
    """Add some more information about parents to the data and export for the /get/:id api"""

    export_data = {}
    for _id, entry in data.items():
        if entry["type"] != "root":
            entry.setdefault("maps", {})["default"] = "interactive"

        entry["aliases"] = []
        if arch_name := extract_arch_name(entry):
            entry["aliases"].append(arch_name)

        export_data[_id] = {
            "parent_names": [data[p]["name"] for p in entry["parents"]],
            **entry,
        }
        if "children" in export_data[_id]:
            del export_data[_id]["children"]
            del export_data[_id]["children_flat"]

        for key in ["tumonline_data", "roomfinder_data", "nat_data"]:
            if key in export_data[_id]:
                del export_data[_id][key]

        if "props" in export_data[_id]:
            prop_keys_to_keep = {"computed", "links", "comment", "calendar_url", "tumonline_room_nr", "operator"}
            to_delete = [e for e in export_data[_id]["props"].keys() if e not in prop_keys_to_keep]
            for k in to_delete:
                del export_data[_id]["props"][k]

    with open(path, "w", encoding="utf-8") as file:
        json.dump(export_data, file, cls=EnhancedJSONEncoder)


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o: Any) -> Any:
        """Enhanced JSONEncoder that can handle dataclasses"""
        if isinstance(o, PydanticConfiguration):
            return o.model_dump()
        return super().default(o)
