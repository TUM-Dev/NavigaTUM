import json
from pathlib import Path

OUTPUT_DIR = Path(__file__).parent.parent / "output"


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
                "parent_building": [data[p]["name"] for p in _data["parents"][building_parents_index:]],
                # For all other parents, only the ids and their keywords (TODO) are searchable
                "parent_keywords": _data["parents"][1:],
                "address": _data.get("tumonline_data", {}).get("address", None),
                "usage": _data.get("usage", {}).get("name", None),
                "rank": int(_data["ranking_factors"]["rank_combined"]),
            },
        )

    with open(path, "w", encoding="utf-8") as file:
        json.dump(export, file)


def export_for_api(data, path):
    """Add some more information about parents to the data and export for the /get/:id api"""
    export_data = {}
    for _id, _data in data.items():
        export_data[_id] = {
            "parent_names": [data[p]["name"] for p in _data["parents"]],
            # "type_common_name": {
            #    "root": "Standortübersicht",
            #    "site": "Standort",
            #    "campus": "Campus",
            #    "area": "Gebiet / Gruppe von Gebäuden",
            #    "joined_building": "Gebäudekomplex",
            #    "building": "Gebäudeteil"
            #                if (_data["type"] == "building" and
            #                    data[_data["parents"][-1]]["type"] == "joined_building")
            #                else "Gebäude",
            #    "room": _data["usage"]["name"] if "usage" in _data else "Raum",
            #    "virtual_room": _data["usage"]["name"] if "usage" in _data else "Raum/Gebäudeteil",
            # }[_data["type"]],
            **_data,
        }
        if "children" in export_data[_id]:
            del export_data[_id]["children"]
            del export_data[_id]["children_flat"]
        if "tumonline_data" in export_data[_id]:
            del export_data[_id]["tumonline_data"]
        if "roomfinder_data" in export_data[_id]:
            del export_data[_id]["roomfinder_data"]
        if "props" in export_data[_id]:
            to_delete = list(filter(lambda e: e != "computed", export_data[_id]["props"].keys()))
            for k in to_delete:
                del export_data[_id]["props"][k]

    with open(path, "w", encoding="utf-8") as file:
        json.dump(export_data, file)
