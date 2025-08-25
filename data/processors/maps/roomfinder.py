import hashlib
import logging
import math
import os.path
from collections import abc
from pathlib import Path
from typing import Any, TypeVar

from external.models import roomfinder
from external.models.common import PydanticConfiguration
from processors.maps.models import Coordinate, CustomBuildingMap, MapKey

BASE = Path(__file__).parent.parent.parent
RESULTS_PATH = BASE / "external" / "results"
RF_MAPS_PATH = RESULTS_PATH / "maps" / "roomfinder"
SOURCES_PATH = BASE / "sources"
CUSTOM_RF_DIR = SOURCES_PATH / "img" / "maps" / "roomfinder"


def assign_roomfinder_maps(data: dict[str, dict[str, Any]]) -> None:
    """Assign roomfinder maps to all entries if there are none yet specified."""
    maps_list = _deduplicate_maps(roomfinder.Map.load_all())
    custom_maps = CustomBuildingMap.load_all()

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
                    if _entry_is_not_on_map(
                        data[child]["coords"],
                        _map.id,
                        _map.width,
                        _map.height,
                        map_assignment_data,
                    ):
                        available_maps.remove(_map)
                        break

        if not available_maps:
            logging.warning(f"No Roomfinder maps available for '{_id}'")
            continue
        _save_map_data(available_maps, entry)


def _save_map_data(available_maps: list[roomfinder.Map], entry: dict[str, Any]) -> None:
    roomfinder_map_data = {
        "available": [
            {
                "id": _map.id,
                "scale": _map.scale,
                "name": _map.desc,
                "width": _map.width,
                "height": _map.height,
                "source": _map.source,
                "file": _map.file,
            }
            for _map in available_maps
        ],
    }
    entry.setdefault("maps", {})["roomfinder"] = roomfinder_map_data


def _set_maps_from_parent(data: dict[str, dict[str, Any]], entry: dict[str, Any]) -> None:
    building_parents: list[str] = [e for e in entry["parents"] if data[e]["type"] == "building"]
    # Verification of this already done for coords.
    # rooms that will not be contained in a parents map will be filtered out in the maps-coverage filter
    building_parent = data[building_parents[0]]
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


def _extract_available_maps(
    entry: dict[str, Any],
    custom_maps: dict[MapKey, roomfinder.Map],
    maps_list: list[roomfinder.Map],
) -> list[roomfinder.Map]:
    """Extract all available maps for the given entry."""
    available_maps: list[roomfinder.Map] = []
    for (b_id, floor), _map in custom_maps.items():
        if (
            entry["type"] == "room"
            and b_id in entry["parents"]
            and "tumonline_data" in entry
            and f".{floor}." in entry["tumonline_data"]["roomcode"]
        ):
            available_maps.append(_map)
    available_maps += maps_list

    def _sort_key(_map: roomfinder.Map) -> tuple[int, float]:
        """Sort by scale and area"""
        scale = int(_map.scale)
        coords = _map.latlonbox
        area = abs(coords.east - coords.west) * abs(coords.north - coords.south)
        return scale, area

    return sorted(available_maps, key=_sort_key)


def _merge_str(s_1: str, s_2: str) -> str:
    """
    Merge two strings. The Result is of the format common_prefix s1/s2 common_suffix.

    Example: "Thierschbau 5. OG" and "Thierschbau 6. OG" -> "Thierschbau 5/6. OG"
    """
    if s_1.strip() == s_2.strip():
        return s_1.strip()
    prefix = os.path.commonprefix((s_1, s_2))
    suffix = os.path.commonprefix((s_1[::-1], s_2[::-1]))[::-1]
    s_1 = s_1.removeprefix(prefix).removesuffix(suffix)
    s_2 = s_2.removeprefix(prefix).removesuffix(suffix)
    if s_1 and s_2:
        return f"{prefix}{s_1}/{s_2}{suffix}"
    # special case: one string is a pre/postfix of the other
    common = s_1 or s_2
    while common.endswith(" "):
        common = common.removesuffix(" ")
        suffix = f" {suffix}"
    while common.startswith(" "):
        common = common.removeprefix(" ")
        prefix = f"{prefix} "
    return f"{prefix}({common.strip()}){suffix}"


MergeMap = TypeVar("MergeMap", bound=dict[str, Any] | type[PydanticConfiguration])


def _merge_maps(map1: MergeMap, map2: MergeMap) -> MergeMap:
    """Merge two Maps into one merged map"""
    result_map = {}
    if isinstance(map1, PydanticConfiguration):
        return map1.__class__.model_validate(_merge_maps(map1.model_dump(), map2.model_dump()))

    for key in map1:
        if key == "id":
            result_map["id"] = map1["id"]
        elif isinstance(map1[key], abc.Mapping):
            result_map[key] = _merge_maps(map1[key], map2[key])
        elif isinstance(map1[key], str):
            result_map[key] = _merge_str(map1[key], map2[key])
        elif isinstance(map1[key], int):
            result_map[key] = int((map1[key] + map2[key]) / 2)
        elif isinstance(map1[key], float):
            result_map[key] = (map1[key] + map2[key]) / 2
        else:
            value = map1[key]
            raise NotImplementedError(f"{key=} ({value=}, {type(value)=}) without a merge-operation defined")
    return result_map


def _deduplicate_maps(maps_list: list[roomfinder.Map]) -> list[roomfinder.Map]:
    """Remove content 1:1 duplicates from the maps_list"""
    content_to_filename_dict: dict[str, str] = {}
    file_renaming_table: dict[str, str] = {}
    for filename in RF_MAPS_PATH.glob("*.webp"):
        file_hash = hashlib.sha256(filename.read_bytes(), usedforsecurity=False).hexdigest()
        _id = filename.with_suffix("").name
        if file_hash in content_to_filename_dict:
            file_renaming_table[_id] = content_to_filename_dict[file_hash]
        else:
            content_to_filename_dict[file_hash] = _id
    # we merge the maps into the first occurrence of said map.
    filtered_map: dict[str, roomfinder.Map] = {_map.id: _map for _map in maps_list}
    for _map in maps_list:
        if _map.id in file_renaming_table:
            map1 = filtered_map.pop(_map.id)
            map2 = filtered_map[file_renaming_table[_map.id]]
            if filtered_map[file_renaming_table[_map.id]] != map1:
                filtered_map[file_renaming_table[_map.id]] = _merge_maps(map1, map2)
    return list(filtered_map.values())


def build_roomfinder_maps(data: dict[str, dict[str, Any]]) -> None:
    """Generate the map information for the Roomfinder maps."""
    map_assignment_data = _generate_assignment_data()

    for entry in data.values():
        if len(entry.get("maps", {}).get("roomfinder", {}).get("available", [])) > 0:
            for entry_map in entry["maps"]["roomfinder"]["available"]:
                x_on_map, y_on_map = _calc_xy_of_coords_on_map(entry["coords"], map_assignment_data[entry_map["id"]])

                entry_map["x"] = x_on_map
                entry_map["y"] = y_on_map

                # set source and filepath so that they are available for all maps
                entry_map.setdefault("source", "Roomfinder")
                entry_map.setdefault("file", f"{entry_map['id']}.webp")


def _calc_xy_of_coords_on_map(coords: Coordinate, map_data: roomfinder.Map) -> tuple[int, int]:
    """
    Calculate the x and y coordinates on a map.

    For the map regions used we can assume that the lat/lon graticule is
    rectangular within that map. It is however not square (roughly 2:3 aspect),
    so for simplicity we first map it into the cartesian pixel coordinate
    system of the image and then apply the rotation.
    Note: x corresponds to longitude, y to latitude
    """
    box = map_data.latlonbox
    box_delta_x = abs(box.west - box.east)
    box_delta_y = abs(box.north - box.south)

    rel_x = abs(box.west - coords["lon"]) / box_delta_x
    rel_y = abs(box.north - coords["lat"]) / box_delta_y

    x0_on_map = rel_x * map_data.width
    y0_on_map = rel_y * map_data.height

    center_x = map_data.width / 2
    center_y = map_data.height / 2

    angle = math.radians(box.rotation)

    float_ix = center_x + (x0_on_map - center_x) * math.cos(angle) - (y0_on_map - center_y) * math.sin(angle)
    float_iy = center_y + (x0_on_map - center_x) * math.sin(angle) + (y0_on_map - center_y) * math.cos(angle)
    return round(float_ix), round(float_iy)


def assign_default_roomfinder_map(data: dict[str, dict[str, Any]]) -> None:
    """Select map with the lowest scale as default"""
    for entry in data.values():
        if rf_maps := entry.get("maps", {}).get("roomfinder"):
            rf_maps.setdefault("default", None)
            if not rf_maps.get("available", None):
                continue

            available_maps = rf_maps["available"]
            default_map = available_maps[0]
            for _map in available_maps:
                if int(_map["scale"]) < int(default_map["scale"]):
                    default_map = _map

            rf_maps["default"] = default_map["id"]


def _generate_assignment_data() -> dict[str, roomfinder.Map]:
    # Read the Roomfinder and custom maps
    return {_map.id: _map for _map in roomfinder.Map.load_all() + list(CustomBuildingMap.load_all().values())}


def _entry_is_not_on_map(
    coords: Coordinate,
    _id: str,
    width: int,
    height: int,
    map_assignment_data: dict[str, roomfinder.Map],
) -> bool:
    # The world map (id rf9) is currently excluded, because it would need a different projection treatment.
    if _id == "rf9":
        return True
    x_on_map, y_on_map = _calc_xy_of_coords_on_map(coords, map_assignment_data[_id])
    x_invalid = x_on_map < 0 or width <= x_on_map
    y_invalid = y_on_map < 0 or height <= y_on_map
    return x_invalid or y_invalid


def remove_non_covering_maps(data: dict[str, dict[str, Any]]) -> None:
    """Remove maps from entries, that do not cover said coordinates"""
    map_assignment_data = _generate_assignment_data()
    for _id, entry in data.items():
        if entry["type"] == "root":
            continue
        if "roomfinder" not in entry["maps"]:
            continue
        rf_maps = entry["maps"]["roomfinder"]
        to_be_deleted = [
            _map
            for _map in rf_maps["available"]
            if _entry_is_not_on_map(entry["coords"], _map["id"], _map["width"], _map["height"], map_assignment_data)
        ]
        for _map in to_be_deleted:
            rf_maps["available"].remove(_map)
        if not rf_maps["available"]:
            # no availible roomfinder maps don't carry any meaning and are deleted
            del entry["maps"]["roomfinder"]
