import json
import logging
import os

from processors import (
    areatree,
    coords,
    images,
    maps,
    merge,
    patch,
    roomfinder,
    search,
    sections,
    sitemap,
    structure,
    tumonline,
)
from utils import convert_to_webp

DEBUG_MODE = "GIT_COMMIT_SHA" not in os.environ.keys()


def main():
    # --- Read base data ---
    logging.info("-- 00 areatree")
    data = areatree.read_areatree()

    logging.info("-- 01 areas extendend")
    data = merge.merge_yaml(data, "sources/01_areas-extended.yaml")

    logging.info("-- 02 rooms extendend")
    data = merge.merge_yaml(data, "sources/02_rooms-extended.yaml")

    # Add source information for these entries, which are up to here
    # always declared by navigatum
    for _id, entry in data.items():
        entry.setdefault("sources", {"base": [{"name": "NavigaTUM"}]})

    # --- Insert Roomfinder and TUMOnline data ---
    logging.info("-- 10 Roomfinder buildings")
    roomfinder.merge_roomfinder_buildings(data)

    logging.info("-- 11 TUMOnline buildings")
    tumonline.merge_tumonline_buildings(data)

    # TUMOnline is used as base and Roomfinder is merged on top of this later
    logging.info("-- 15 TUMOnline rooms")
    tumonline.merge_tumonline_rooms(data)

    logging.info("-- 16 Roomfinder rooms")
    roomfinder.merge_roomfinder_rooms(data)

    # At this point, no more areas or rooms will be added or removed.
    # --- Make data more coherent ---
    logging.info("-- 30 Add children properties")
    structure.add_children_properties(data)

    logging.info("-- 33 Add (structural) stats")
    structure.add_stats(data)

    logging.info("-- 34 Infer more props")
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
            if (_data["type"] == "building" and data[_data["parents"][-1]]["type"] == "joined_building")
            else "Gebäude",
            "room": _data["usage"]["name"] if "usage" in _data else "Raum",
            "virtual_room": _data["usage"]["name"] if "usage" in _data else "Raum/Gebäudeteil",
        }[_data["type"]]

    logging.info("-- 40 Coordinates")
    coords.assert_buildings_have_coords(data)
    coords.assign_coordinates(data)
    coords.check_coords(data)

    logging.info("-- 45 Roomfinder maps")
    maps.assign_roomfinder_maps(data)
    maps.remove_non_covering_maps(data)
    maps.assign_default_roomfinder_map(data)
    maps.build_roomfinder_maps(data)

    logging.info("-- 46 Overlay maps")
    maps.add_overlay_maps(data)

    logging.info(f"-- 50 convert {images.IMAGE_BASE} to webp")
    convert_to_webp(images.IMAGE_BASE)
    logging.info("-- 51 resize and crop the images for different resolutions and formats")
    images.resize_and_crop()
    logging.info("-- 52 Add image information")
    images.add_img(data)

    logging.info("-- 80 Generate info card")
    sections.compute_props(data)

    logging.info("-- 81 Generate overview sections")
    sections.generate_buildings_overview(data)
    sections.generate_rooms_overview(data)

    logging.info("-- 90 Search: Build base ranking")
    search.add_ranking_base(data)

    logging.info("-- 97 Search: Get combined ranking")
    search.add_ranking_combined(data)

    logging.info("-- 99 Search: Export")
    export_for_search(data, "output/search_data.json")

    for _id, entry in data.items():
        if entry["type"] != "root":
            entry.setdefault("maps", {})["default"] = "interactive"

    logging.info("-- 100 Export: API")
    export_for_api(data, "output/api_data.json")

    # Sitemap is only generated for deployment:
    if DEBUG_MODE:
        logging.info("Skipping sitemap generation in Dev Mode (GIT_COMMIT_SHA is unset)")
    else:
        logging.info("-- 101 Extra: Sitemap")
        sitemap.generate_sitemap()


def export_for_search(data, path):
    export = []
    for _id, _data in data.items():
        # Currently, the "root" entry is excluded from search
        if _id == "root":
            continue

        if _data["type"] in {"room", "virtual_room"}:
            building_parents_index = list(
                map(
                    lambda e: data[e]["type"] in {"building", "joined_building"},
                    _data["parents"],
                ),
            ).index(True)
        else:
            building_parents_index = len(_data["parents"])

        export.append(
            {
                "ms_id": _id.replace(
                    ".",
                    "-",
                ),  # MeiliSearch requires an id without "."; also this puts more emphasis on the order (because "." counts as more distance)
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
            },
        )

    with open(path, "w") as f:
        json.dump(export, f)


def export_for_api(data, path):
    # Add some more information about parents
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

    with open(path, "w") as f:
        json.dump(export_data, f)


if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG if DEBUG_MODE else logging.INFO, format="%(levelname)s: %(message)s")
    logging.addLevelName(logging.INFO, f"\033[1;36m{logging.getLevelName(logging.INFO)}\033[1;0m")
    logging.addLevelName(logging.WARNING, f"\033[1;33m{logging.getLevelName(logging.WARNING)}\033[1;0m")
    logging.addLevelName(logging.ERROR, f"\033[1;41m{logging.getLevelName(logging.ERROR)}\033[1;0m")
    logging.addLevelName(logging.CRITICAL, f"\033[1;41m{logging.getLevelName(logging.CRITICAL)}\033[1;0m")

    # Pillow prints all imported modules to the debug stream
    logging.getLogger("PIL").setLevel(logging.INFO)
    main()
