import logging
import math
from pathlib import Path
from typing import Any

from external.models import roomfinder
from external.models.roomfinder import LatLonBox, Coordinate
from processors.maps.models import CustomBuildingMap, MapKey

BASE_PATH = Path(__file__).parent.parent.parent
RESULTS_PATH = BASE_PATH / "external" / "results"
SITE_PLANS_PATH = BASE_PATH / "sources" / "img" / "maps" / "site_plans"
SOURCES_PATH = BASE_PATH / "sources"


def assign_roomfinder_maps(data: dict[str, dict[str, Any]]) -> None:
    """Assign roomfinder maps to all entries if there are none yet specified."""
    maps_list = roomfinder.Map.load_all()
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


def build_roomfinder_maps(data: dict[str, dict[str, Any]]) -> None:
    """Generate the map information for the Roomfinder maps."""
    map_assignment_data = _generate_assignment_data()

    for entry in data.values():
        if len(entry.get("maps", {}).get("roomfinder", {}).get("available", [])) > 0:
            for entry_map in entry["maps"]["roomfinder"]["available"]:
                assign_map = map_assignment_data[entry_map["id"]]
                x_on_map, y_on_map = _calc_xy_of_coords_on_map(
                    entry["coords"], assign_map.latlonbox, assign_map.width, assign_map.width
                )

                entry_map["x"] = x_on_map
                entry_map["y"] = y_on_map

                # set source and filepath so that they are available for all maps
                # custom maps have the source already set
                entry_map.setdefault("source", "Roomfinder")
                entry_map.setdefault("file", f"{entry_map['id']}.webp")


def _calc_xy_of_coords_on_map(
        coords: Coordinate, map_latlonbox: LatLonBox, map_width: int, map_height: int
) -> tuple[int, int] | None:
    """
    Calculate the x and y coordinates on a map.

    For the map regions used we can assume that the lat/lon graticule is
    rectangular within that map. It is however not square (roughly 2:3 aspect),
    so for simplicity we first map it into the cartesian pixel coordinate
    system of the image and then apply the rotation.
    Note: x corresponds to longitude, y to latitude
    """
    if coords not in map_latlonbox:
        return None

    box_delta_x = abs(map_latlonbox.west - map_latlonbox.east)
    box_delta_y = abs(map_latlonbox.north - map_latlonbox.south)

    rel_x = abs(map_latlonbox.west - coords["lon"]) / box_delta_x
    rel_y = abs(map_latlonbox.north - coords["lat"]) / box_delta_y

    x0_on_map = rel_x * map_width
    y0_on_map = rel_y * map_height

    center_x = map_width / 2
    center_y = map_height / 2

    angle = math.radians(map_latlonbox.rotation)

    float_ix = center_x + (x0_on_map - center_x) * math.cos(angle) - (y0_on_map - center_y) * math.sin(angle)
    float_iy = center_y + (x0_on_map - center_x) * math.sin(angle) + (y0_on_map - center_y) * math.cos(angle)

    x_valid = 0 <= float_ix <= map_width
    y_valid = 0 <= float_iy <= map_height
    return (round(float_ix), round(float_iy)) if x_valid and y_valid else None


def assign_default_roomfinder_map(data: dict[str, dict[str, Any]]) -> None:
    """Select map with the lowest scale as default"""
    for entry in data.values():
        if rf_maps := entry.get("maps", {}).get("roomfinder"):
            rf_maps.setdefault("default", None)
            if available_maps := rf_maps.get("available", None):
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
        map_id: str,
        width: int,
        height: int,
        map_assignment_data: dict[str, roomfinder.Map],
) -> bool:
    assign_map = map_assignment_data[map_id]
    x_on_map, y_on_map = _calc_xy_of_coords_on_map(coords, assign_map.latlonbox, assign_map.width, assign_map.width)
    x_invalid = x_on_map < 0 or width <= x_on_map
    y_invalid = y_on_map < 0 or height <= y_on_map
    return x_invalid or y_invalid


def remove_non_covering_maps(data: dict[str, dict[str, Any]]) -> None:
    """Remove maps from entries, that do not cover said coordinates"""
    map_assignment_data = _generate_assignment_data()
    for _id, entry in data.items():
        if entry["type"] == "root":
            continue
        if rf_maps := entry["maps"].get("roomfinder"):
            to_be_deleted = [
                _map
                for _map in rf_maps["available"]
                if _entry_is_not_on_map(entry["coords"], _map["id"], _map["width"], _map["height"], map_assignment_data)
            ]
            for _map in to_be_deleted:
                rf_maps["available"].remove(_map)
            if not rf_maps["available"]:
                # no available roomfinder maps don't carry any meaning and are deleted
                del entry["maps"]["roomfinder"]
