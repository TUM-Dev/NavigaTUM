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
            for m in available_maps:
                for c in entry["children"]:
                    if _entry_is_not_on_map(data[c], m, map_assignment_data):
                        available_maps.remove(m)
                        break

        if not available_maps:
            logging.warning(f"No Roomfinder maps available for '{_id}'")
            continue
        _save_map_data(available_maps, entry)


def _save_map_data(available_maps, entry):
    roomfinder_map_data = {
        "available": [
            {
                "id": m["id"],
                "scale": m["scale"],
                "name": m["desc"],
                "width": m["width"],
                "height": m["height"],
                "source": m.get("source", "Roomfinder"),
                "file": m.get("file", f"{m['id']}.webp"),
            }
            for m in available_maps
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
    for (b_id, floor), m in custom_maps.items():
        if (
            entry["type"] == "room"
            and b_id in entry["parents"]
            and "tumonline_data" in entry
            and f".{floor}." in entry["tumonline_data"]["roomcode"]
        ):
            available_maps.append(m)
    available_maps += maps_list

    def _sort_key(_map):
        """sort by scale and area"""
        scale = int(_map["scale"])
        coords = _map["latlonbox"]
        area = abs(coords["east"] - coords["west"]) * abs(coords["north"] - coords["south"])
        return scale, area

    available_maps.sort(key=_sort_key)

    return available_maps


def _merge_str(s1: str, s2: str):
    """
    Merges two strings. The Result is of the format common_prefix s1/s2 common_suffix.
    Example: "Thierschbau 5. OG" and "Thierschbau 6. OG" -> "Thierschbau 5/6. OG"
    """
    if s1 == s2:
        return s1
    prefix = os.path.commonprefix((s1, s2))
    suffix = os.path.commonprefix((s1[::-1], s2[::-1]))[::-1]
    s1 = s1.removeprefix(prefix).removesuffix(suffix)
    s2 = s2.removeprefix(prefix).removesuffix(suffix)
    return prefix + s1 + "/" + s2 + suffix


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
    file_renaming_table: dict[str, str] = dict()
    for filename in RF_MAPS_PATH.iterdir():
        with open(filename, "rb") as file:
            content = file.read()
            _id = filename.with_suffix("").name
            if content in content_to_filename_dict:
                file_renaming_table[_id] = content_to_filename_dict[content]
            else:
                content_to_filename_dict[content] = _id
    # we merge the maps into the first occurrence of said map.
    filtered_map = {m["id"]: m for m in maps_list}
    for m in maps_list:
        _id = m["id"]
        if _id in file_renaming_table:
            map1 = filtered_map.pop(_id)
            map2 = filtered_map[file_renaming_table[_id]]
            if filtered_map[file_renaming_table[_id]] != map1:
                filtered_map[file_renaming_table[_id]] = _merge_maps(map1, map2)
    maps_list = list(filtered_map.values())
    return maps_list


def _load_maps_list():
    """Read the Roomfinder maps. The world-map is not used"""
    with open("external/maps_roomfinder.json") as f:
        maps_list: list[dict[str, Any]] = json.load(f)
    world_map = None
    for m in maps_list:
        if m["id"] == 9:  # World map is not used
            world_map = m
        else:
            m["latlonbox"]["north"] = float(m["latlonbox"]["north"])
            m["latlonbox"]["south"] = float(m["latlonbox"]["south"])
            m["latlonbox"]["east"] = float(m["latlonbox"]["east"])
            m["latlonbox"]["west"] = float(m["latlonbox"]["west"])
            m["id"] = f"rf{m['id']}"
    maps_list.remove(world_map)

    # remove 1:1 content duplicates
    maps_list = _deduplicate_maps(maps_list)

    return maps_list


def build_roomfinder_maps(data):
    """Generate the map information for the Roomfinder maps."""

    map_assignment_data = _generate_assignment_data()

    for _id, entry in data.items():
        if len(entry.get("maps", {}).get("roomfinder", {}).get("available", [])) > 0:
            for entry_map in entry["maps"]["roomfinder"]["available"]:
                map_data = map_assignment_data[entry_map["id"]]
                ix, iy = _calc_xy_of_coords_on_map(entry["coords"], map_data)

                entry_map["x"] = ix
                entry_map["y"] = iy

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

    ix0: float = rel_x * map_data["width"]
    iy0: float = rel_y * map_data["height"]

    cx: float = map_data["width"] / 2
    cy: float = map_data["height"] / 2

    angle: float = math.radians(float(box["rotation"]))

    ix: float = cx + (ix0 - cx) * math.cos(angle) - (iy0 - cy) * math.sin(angle)
    iy: float = cy + (ix0 - cx) * math.sin(angle) + (iy0 - cy) * math.cos(angle)
    int_ix, int_iy = round(ix), round(iy)
    return int_ix, int_iy


def _load_custom_maps():
    """Load the custom maps like Roomfinder maps"""
    with open("sources/45_custom-maps.yaml") as f:
        custom_maps = yaml.safe_load(f.read())

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
    with open("sources/46_overlay-maps.yaml") as f:
        overlay_maps = yaml.safe_load(f.read())

    parent_lut = {m["props"]["parent"]: m for m in overlay_maps}
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
            for m in overlay["maps"]:
                overlay_data["available"].append(
                    {
                        "id": m["id"],
                        "floor": m["floor"],
                        "file": m["file"],
                        "name": m["desc"],
                        "coordinates": overlay["props"]["box"],
                    },
                )

                # The 'tumonline' field overwrites which TUMOnline ID floor to match
                if f".{m.get('tumonline', '')}." in _id:
                    overlay_data["default"] = m["id"]
                elif overlay_data.get("default", None) is None and f".{m['floor']}." in _id:
                    overlay_data["default"] = m["id"]

            overlay_data.setdefault("default", None)


def assign_default_roomfinder_map(data):
    """Selects map with lowest scale as default"""
    for _id, entry in data.items():
        if "maps" in entry and "roomfinder" in entry["maps"]:
            rf = entry["maps"]["roomfinder"]
            rf.setdefault("default", None)
            if not rf.get("available", None):
                continue

            available_maps = rf["available"]
            default_map = available_maps[0]
            for m in available_maps:
                if int(m["scale"]) < int(default_map["scale"]):
                    default_map = m

            rf["default"] = default_map["id"]


def _generate_assignment_data():
    # Read the Roomfinder and custom maps
    with open("external/maps_roomfinder.json") as f:
        maps_list = json.load(f)
    custom_maps = _load_custom_maps()
    # For each map, we calculate the boundaries in UTM beforehand
    map_assignment_data = {}
    for m in maps_list + list(custom_maps.values()):
        if "latlonbox" in m:
            # Roomfinder data is with ints as id, but we use a string based format
            if isinstance(m["id"], int):
                m["id"] = f"rf{m['id']}"

            map_assignment_data[m["id"]] = m
    return map_assignment_data


def _entry_is_not_on_map(entry, m, map_assignment_data):
    map_id = m["id"]
    # The world map (id 9) is currently excluded, because it would need a different
    # projection treatment.
    if map_id == "rf9":
        return True
    ix, iy = _calc_xy_of_coords_on_map(entry["coords"], map_assignment_data[map_id])
    return not ((0 <= ix < m["width"]) and (0 <= iy < m["height"]))


def remove_non_covering_maps(data):
    map_assignment_data = _generate_assignment_data()
    for _id, entry in data.items():
        if entry["type"] == "root":
            continue
        if "roomfinder" not in entry["maps"]:
            continue
        rf = entry["maps"]["roomfinder"]
        to_be_deleted = []
        for m in rf["available"]:
            if _entry_is_not_on_map(entry, m, map_assignment_data):
                to_be_deleted.append(m)
        for m in to_be_deleted:
            rf["available"].remove(m)
        if not rf["available"]:
            # no availible roomfinder maps dont carry any meaning and are deleted
            del entry["maps"]["roomfinder"]
