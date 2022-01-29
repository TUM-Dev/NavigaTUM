import json

from processors import areatree, check, images, maps, merge, patch, roomfinder, search, sections, structure, tumonline


def main():
    # --- Read base data ---
    print("-- 00 areatree")
    data = areatree.read_areatree()

    print("-- 01 areas extendend")
    data = merge.merge_yaml(data, "sources/01_areas-extended.yaml")
    
    print("-- 02 rooms extendend")
    data = merge.merge_yaml(data, "sources/02_rooms-extended.yaml")

    # --- Insert Roomfinder and TUMOnline data ---
    print("-- 10 Roomfinder buildings")
    roomfinder.merge_roomfinder_buildings(data)

    print("-- 11 TUMOnline buildings")
    tumonline.merge_tumonline_buildings(data)
    
    # TUMOnline is used as base and Roomfinder is merged on top of this later
    print("-- 15 TUMOnline rooms")
    tumonline.merge_tumonline_rooms(data)
    
    print("-- 16 Roomfinder rooms")
    roomfinder.merge_roomfinder_rooms(data)

    # At this point, no more areas or rooms will be added or removed.
    # --- Make data more coherent ---
    print("-- 30 Add children properties")
    structure.add_children_properties(data)
    
    print("-- 33 Add (structural) stats")
    structure.add_stats(data)
    
    print("-- 34 Infer more props")
    structure.infer_addresses(data)
    
    # TODO: Does it make sense to introduce a type 'sub_building' here?

    for _id, _data in data.items():
        _data["type_common_name"] = {
            "root": "Standortübersicht",
            "site": "Standort",
            "campus": "Campus",
            "area": "Gebiet / Gruppe von Gebäuden",
            "joined_building": "Gebäudekomplex",
            "building": "Gebäudeteil"
                        if (_data["type"] == "building" and
                            data[_data["parents"][-1]]["type"] == "joined_building")
                        else "Gebäude",
            "room": _data["usage"]["name"] if "usage" in _data else "Raum",
            "virtual_room": _data["usage"]["name"] if "usage" in _data else "Raum/Gebäudeteil",
        }[_data["type"]]

    print("-- 40 Coordinates")
    #check.check_coords(data)
    maps.assign_coordinates(data)
    
    print("-- 45 Roomfinder maps")
    maps.assign_roomfinder_maps(data)
    maps.build_roomfinder_maps(data)
    
    print("-- 50 Add image information")
    images.add_img(data, "sources/img/")
    
    print("-- 80 Generate info card")
    sections.compute_props(data)
    
    print("-- 81 Generate overview sections")
    sections.generate_buildings_overview(data)
    sections.generate_rooms_overview(data)
    
    print("-- 90 Search: Build base ranking")
    search.add_ranking_base(data)
    
    print("-- 97 Search: Get combined ranking")
    search.add_ranking_combined(data)
    
    print("-- 99 Search: Export")
    export_for_search(data, "output/search_data.json")
    
    for _id, entry in data.items():
        if entry["type"] != "root":
            entry.setdefault("maps", {})["default"] = "interactive"
    
    print("-- 100 Export: API")
    export_for_api(data, "output/api_data.json")


def export_for_search(data, path):
    export = []
    for _id, _data in data.items():
        # Currently, the "root" entry is excluded from search
        if _id == "root":
            continue

        if _data["type"] in {"room", "virtual_room"}:
            building_parents_index = list(
                map(lambda e: data[e]["type"] in {"building", "joined_building"},
                    _data["parents"])
            ).index(True)
        else:
            building_parents_index = len(_data["parents"])

        export.append({
            "ms_id": _id.replace(".", "-"), # MeiliSearch requires an id without "."; also this puts more emphasis on the order (because "." counts as more distance)
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
            ###"parent_names": _data["parents"][1:],#[data[p]["name"] for p in _data["parents"][1:]],
            # For rooms, the (joined_)building parents are extra to put more emphasis on them.
            # Also their name is included
            "parent_building": [data[p]["name"] for p in _data["parents"][building_parents_index:]],
            # For all other parents, only the ids and their keywords (TODO) are searchable
            "parent_keywords": _data["parents"][1:],
            "address": _data.get("tumonline_data", {}).get("address", None),
            "usage": _data.get("usage", {}).get("name", None),
            "rank": int(_data["ranking_factors"]["rank_combined"]),
        })
    
    with open(path, "w") as f:
        json.dump(export, f)


def export_for_api(data, path):
    # Add some more information about parents
    export_data = {}
    for _id, _data in data.items():
        export_data[_id] = {
            "parent_names": [data[p]["name"] for p in _data["parents"]],
            #"type_common_name": {
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
            #}[_data["type"]],
            **_data
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
    
    with open(path, "w") as f:
        json.dump(export_data, f)
    


if __name__ == "__main__":
    main()
