import json
import logging
import math
import os.path
from pathlib import Path
from typing import Any

import yaml
from PIL import Image

EXTERNAL_PATH = Path(__file__).parent.parent / "external"
RF_MAPS_PATH = EXTERNAL_PATH / "maps" / "roomfinder"


def assign_roomfinder_maps(data):
    """
    Assign roomfinder maps to all entries if there are none yet specified.
    """
    maps_list = _load_maps_list()

    # There are also Roomfinder-like custom maps, that we assign here
    custom_maps = _load_custom_maps()

    # further data about each map, only used for map-assignment
    map_assignment_data = _generate_assignment_data()

    for _id, entry in data.items():
        if entry["type"] == "root":
            continue

        if entry.get("maps", {}).get("roomfinder", {}).get("available", []):
            continue

        # Use maps from parent building, if there is no precise coordinate known
        entry_accuracy_building = entry["coords"].get("accuracy", None) == "building"
        if entry_accuracy_building and entry["type"] in {"room", "virtual_room"}:
            _set_maps_from_parent(data, entry)
            continue

        available_maps = _extract_available_maps(entry, custom_maps, maps_list)

        # For entries of these types only show maps that contain all (direct) children.
        # This is to make sure that only (high scale) maps are included here that make sense.
        # TODO: zentralgelaende
        if entry["type"] in {"site", "campus", "area", "joined_building", "building"} and "children" in entry:
            for _map in available_maps:
                for child in entry["children"]:
                    if _entry_is_not_on_map(data[child], _map, map_assignment_data):
                        available_maps.remove(_map)
                        break

        if not available_maps:
            logging.warning(f"No Roomfinder maps available for '{_id}'")
            continue
        _save_map_data(available_maps, entry)


def _save_map_data(available_maps, entry):
    roomfinder_map_data = {
        "available": [
            {
                "id": _map["id"],
                "scale": _map["scale"],
                "name": _map["desc"],
                "width": _map["width"],
                "height": _map["height"],
                "source": _map.get("source", "Roomfinder"),
                "file": _map.get("file", f"{_map['id']}.webp"),
            }
            for _map in available_maps
        ],
    }
    entry.setdefault("maps", {})["roomfinder"] = roomfinder_map_data


def _set_maps_from_parent(data, entry):
    building_parent = [e for e in entry["parents"] if data[e]["type"] == "building"]
    # Verification of this already done for coords.
    # rooms that will not be contained in a parents map will be filtered out in the maps-coverage filter
    building_parent = data[building_parent[0]]
    if building_parent.get("maps", {}).get("roomfinder", {}).get("available", []):
        # TODO: 5510.02.001
        roomfinder_map_data = entry.setdefault("maps", {}).get("roomfinder", {})
        roomfinder_map_data.update(building_parent["maps"]["roomfinder"])
        roomfinder_map_data["is_only_building"] = True
    else:
        roomfinder_map_data = {"is_only_building": True}
        # Both share the reference now, assuming that the parening_parent
        # will be processed some time later in this loop.
        building_parent.setdefault("maps", {})["roomfinder"] = roomfinder_map_data
        entry.setdefault("maps", {})["roomfinder"] = roomfinder_map_data


def _extract_available_maps(entry, custom_maps, maps_list):
    available_maps = []
    for (b_id, floor), _map in custom_maps.items():
        if (
            entry["type"] == "room"
            and b_id in entry["parents"]
            and "tumonline_data" in entry
            and f".{floor}." in entry["tumonline_data"]["roomcode"]
        ):
            available_maps.append(_map)
    available_maps += maps_list

    def _sort_key(_map):
        """sort by scale and area"""
        scale = int(_map["scale"])
        coords = _map["latlonbox"]
        area = abs(coords["east"] - coords["west"]) * abs(coords["north"] - coords["south"])
        return scale, area

    available_maps.sort(key=_sort_key)

    return available_maps


def _merge_str(s_1: str, s_2: str):
    """
    Merges two strings. The Result is of the format common_prefix s1/s2 common_suffix.
    Example: "Thierschbau 5. OG" and "Thierschbau 6. OG" -> "Thierschbau 5/6. OG"
    """
    if s_1 == s_2:
        return s_1
    prefix = os.path.commonprefix((s_1, s_2))
    suffix = os.path.commonprefix((s_1[::-1], s_2[::-1]))[::-1]
    s_1 = s_1.removeprefix(prefix).removesuffix(suffix)
    s_2 = s_2.removeprefix(prefix).removesuffix(suffix)
    return prefix + s_1 + "/" + s_2 + suffix


def _merge_maps(map1, map2):
    """Merges two Maps into one merged map"""
    result_map = {}
    for key in map1.keys():
        if key == "id":
            result_map["id"] = map1["id"]
        elif isinstance(map1[key], dict):
            result_map[key] = _merge_maps(map1[key], map2[key])
        elif isinstance(map1[key], str):
            result_map[key] = _merge_str(map1[key], map2[key])
        elif isinstance(map1[key], int):
            result_map[key] = int((map1[key] + map2[key]) / 2)
        elif isinstance(map1[key], float):
            result_map[key] = (map1[key] + map2[key]) / 2
        else:
            values = map1[key]
            raise NotImplementedError(f"the {key=} of with {type(values)=} does not have a merging-operation defined")
    return result_map


def _deduplicate_maps(maps_list):
    """Remove content 1:1 duplicates from the maps_list"""
    content_to_filename_dict = {}
    file_renaming_table: dict[str, str] = {}
    for filename in RF_MAPS_PATH.iterdir():
        with open(filename, "rb") as file:
            content = file.read()
            _id = filename.with_suffix("").name
            if content in content_to_filename_dict:
                file_renaming_table[_id] = content_to_filename_dict[content]
            else:
                content_to_filename_dict[content] = _id
    # we merge the maps into the first occurrence of said map.
    filtered_map = {_map["id"]: _map for _map in maps_list}
    for _map in maps_list:
        _id = _map["id"]
        if _id in file_renaming_table:
            map1 = filtered_map.pop(_id)
            map2 = filtered_map[file_renaming_table[_id]]
            if filtered_map[file_renaming_table[_id]] != map1:
                filtered_map[file_renaming_table[_id]] = _merge_maps(map1, map2)
    return list(filtered_map.values())


def _load_maps_list():
    """Read the Roomfinder maps. The world-map is not used"""
    with open("external/maps_roomfinder.json", encoding="utf-8") as file:
        maps_list: list[dict[str, Any]] = json.load(file)
    world_map = None
    for _map in maps_list:
        if _map["id"] == 9:  # World map is not used
            world_map = _map
        else:
            _map["latlonbox"]["north"] = float(_map["latlonbox"]["north"])
            _map["latlonbox"]["south"] = float(_map["latlonbox"]["south"])
            _map["latlonbox"]["east"] = float(_map["latlonbox"]["east"])
            _map["latlonbox"]["west"] = float(_map["latlonbox"]["west"])
            _map["id"] = f"rf{_map['id']}"
    maps_list.remove(world_map)

    # remove 1:1 content duplicates
    return _deduplicate_maps(maps_list)


def build_roomfinder_maps(data):
    """Generate the map information for the Roomfinder maps."""

    map_assignment_data = _generate_assignment_data()

    for _id, entry in data.items():
        if len(entry.get("maps", {}).get("roomfinder", {}).get("available", [])) > 0:
            for entry_map in entry["maps"]["roomfinder"]["available"]:
                map_data = map_assignment_data[entry_map["id"]]
                x_on_map, y_on_map = _calc_xy_of_coords_on_map(entry["coords"], map_data)

                entry_map["x"] = x_on_map
                entry_map["y"] = y_on_map

                # set source and filepath so that they are available for all maps
                entry_map.setdefault("source", "Roomfinder")
                entry_map.setdefault("file", f"{entry_map['id']}.webp")


def _calc_xy_of_coords_on_map(coords, map_data) -> tuple[int, int]:
    """
    For the map regions used we can assume that the lat/lon graticule is
    rectangular within that map. It is however not square (roughly 2:3 aspect),
    so for simplicity we first map it into the cartesian pixel coordinate
    system of the image and then apply the rotation.
    Note: x corresponds to longitude, y to latitude
    """
    box = map_data["latlonbox"]
    entry_x, entry_y = coords["lon"], coords["lat"]
    box_delta_x: float = abs(float(box["west"]) - float(box["east"]))
    box_delta_y: float = abs(float(box["north"]) - float(box["south"]))

    rel_x: float = abs(float(box["west"]) - entry_x) / box_delta_x
    rel_y: float = abs(float(box["north"]) - entry_y) / box_delta_y

    x0_on_map: float = rel_x * map_data["width"]
    y0_on_map: float = rel_y * map_data["height"]

    center_x: float = map_data["width"] / 2
    center_y: float = map_data["height"] / 2

    angle: float = math.radians(float(box["rotation"]))

    ix: float = center_x + (x0_on_map - center_x) * math.cos(angle) - (y0_on_map - center_y) * math.sin(angle)
    iy: float = center_y + (x0_on_map - center_x) * math.sin(angle) + (y0_on_map - center_y) * math.cos(angle)
    int_ix, int_iy = round(ix), round(iy)
    return int_ix, int_iy


def _load_custom_maps():
    """Load the custom maps like Roomfinder maps"""
    with open("sources/45_custom-maps.yaml", encoding="utf-8") as file:
        custom_maps = yaml.safe_load(file.read())

    # Convert into the format used by maps_roomfinder.json:
    maps_out = {}
    for map_group in custom_maps:
        base_data = {
            "source": map_group["props"].get("source", "NavigaTUM-Contributors"),
            # For some reason, these are given as str
            "scale": str(map_group["props"]["scale"]),
            "latlonbox": {
                "north": map_group["props"]["north"],
                "east": map_group["props"]["east"],
                "west": map_group["props"]["west"],
                "south": map_group["props"]["south"],
                "rotation": map_group["props"]["rotation"],
            },
        }
        for sub_map in map_group["maps"]:
            img = Image.open("sources/img/maps/roomfinder/" + sub_map["file"])
            maps_out[(sub_map["b_id"], sub_map["floor"])] = {
                "desc": sub_map["desc"],
                "id": ".".join(sub_map["file"].split(".")[:-1]),
                "file": sub_map["file"],
                "width": img.width,
                "height": img.height,
                **base_data,
            }

    return maps_out


def add_overlay_maps(data):
    """Add the overlay maps to all entries where they apply"""
    with open("sources/46_overlay-maps.yaml", encoding="utf-8") as file:
        overlay_maps = yaml.safe_load(file.read())

    parent_lut = {_map["props"]["parent"]: _map for _map in overlay_maps}
    parent_ids = set(parent_lut.keys())

    for _id, entry in data.items():
        candidates = parent_ids.intersection(entry["parents"])
        if len(candidates) > 1:
            logging.warning(
                f"Multiple candidates as overlay map for {_id}: {candidates}. "
                f"Currently this is not supported! Skipping ...",
            )
        elif bool(candidates) ^ (_id in parent_ids):
            # either a candidate exist or _id is one of the parent ids, but not both
            overlay = parent_lut[list(candidates)[0] if len(candidates) == 1 else _id]
            overlay_data = entry.setdefault("maps", {}).setdefault("overlays", {})
            overlay_data["available"] = []
            for _map in overlay["maps"]:
                overlay_data["available"].append(
                    {
                        "id": _map["id"],
                        "floor": _map["floor"],
                        "file": _map["file"],
                        "name": _map["desc"],
                        "coordinates": overlay["props"]["box"],
                    },
                )

                # The 'tumonline' field overwrites which TUMOnline ID floor to match
                if (f".{_map.get('tumonline', '')}." in _id) or (
                    overlay_data.get("default", None) is None and f".{_map['floor']}." in _id
                ):
                    overlay_data["default"] = _map["id"]

            overlay_data.setdefault("default", None)


def assign_default_roomfinder_map(data):
    """Selects map with lowest scale as default"""
    for _id, entry in data.items():
        if "maps" in entry and "roomfinder" in entry["maps"]:
            rf_maps = entry["maps"]["roomfinder"]
            rf_maps.setdefault("default", None)
            if not rf_maps.get("available", None):
                continue

            available_maps = rf_maps["available"]
            default_map = available_maps[0]
            for _map in available_maps:
                if int(_map["scale"]) < int(default_map["scale"]):
                    default_map = _map

            rf_maps["default"] = default_map["id"]


def _generate_assignment_data():
    # Read the Roomfinder and custom maps
    with open("external/maps_roomfinder.json", encoding="utf-8") as file:
        maps_list = json.load(file)
    custom_maps = _load_custom_maps()
    # For each map, we calculate the boundaries in UTM beforehand
    map_assignment_data = {}
    for _map in maps_list + list(custom_maps.values()):
        if "latlonbox" in _map:
            # Roomfinder data is with ints as id, but we use a string based format
            if isinstance(_map["id"], int):
                _map["id"] = f"rf{_map['id']}"

            map_assignment_data[_map["id"]] = _map
    return map_assignment_data


def _entry_is_not_on_map(entry, _map, map_assignment_data):
    map_id = _map["id"]
    # The world map (id 9) is currently excluded, because it would need a different
    # projection treatment.
    if map_id == "rf9":
        return True
    x_on_map, y_on_map = _calc_xy_of_coords_on_map(entry["coords"], map_assignment_data[map_id])
    x_invalid = 0 > x_on_map or _map["width"] <= x_on_map
    y_invalid = 0 > y_on_map or _map["height"] <= y_on_map
    return x_invalid or y_invalid


def remove_non_covering_maps(data):
    """Removes maps from entries, that do not cover said coordinates"""
    map_assignment_data = _generate_assignment_data()
    for _id, entry in data.items():
        if entry["type"] == "root":
            continue
        if "roomfinder" not in entry["maps"]:
            continue
        roomfinder = entry["maps"]["roomfinder"]
        to_be_deleted = []
        for _map in roomfinder["available"]:
            if _entry_is_not_on_map(entry, _map, map_assignment_data):
                to_be_deleted.append(_map)
        for _map in to_be_deleted:
            roomfinder["available"].remove(_map)
        if not roomfinder["available"]:
            # no availible roomfinder maps dont carry any meaning and are deleted
            del entry["maps"]["roomfinder"]
