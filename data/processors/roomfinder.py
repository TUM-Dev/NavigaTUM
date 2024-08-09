import logging
import re
from pathlib import Path
from typing import Any

import yaml
from external.models import roomfinder

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"


def merge_roomfinder_buildings(data: dict[str, dict[str, Any]]) -> None:
    """
    Merge the buildings in Roomfinder with the existing data.

    This will not overwrite the existing data, but act directly on the provided data.
    """
    with (SOURCES_PATH / "10_patches-roomfinder-buildings.yaml").open(encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    error = False
    for building in roomfinder.Building.load_all():
        # 'Building' 0000 contains some buildings and places not in TUMonline as rooms.
        # They might be integrated customly somewhere else, but here we ignore these.
        if building.b_id == "0000":
            continue

        for wrong, correct in patches["replacements"].items():
            if building.b_id == wrong:
                building.b_id = correct

        # Find the corresponding building in the existing data
        internal_id = None
        for _id, _data in data.items():
            if "b_prefix" in _data and _data["b_prefix"] == building.b_id:
                if internal_id is None:
                    internal_id = _id
                else:
                    logging.error(f"building id '{building.b_id}' more than once in base data")
                    error = True
                    break

        if internal_id is None:
            # The Roomfinder appears to be no longer maintained, so sometimes there are still
            # buildings in it that no longer exist. Previously this was an error, but for this
            # reason now it is a warning.
            logging.warning(f"building '{building.b_id}' not found in base data. It may be missing in the areatree.")
            continue

        b_data = data[internal_id]

        b_data["roomfinder_data"] = {
            "b_id": building.b_id,
            "b_name": building.b_name,
            "b_alias": building.b_alias,
            "b_area": building.b_area,
            "b_room_count": building.b_room_count,
        }

        b_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "Roomfinder",
                "url": f"https://portal.mytum.de/displayRoomMap?@{building.b_id}",
            },
        )

        b_data.setdefault(
            "coords",
            {
                "lat": building.lat,
                "lon": building.lon,
                "source": "roomfinder",
            },
        )
        if building.maps:
            b_data.setdefault("maps", {})["roomfinder"] = _get_roomfinder_maps(building)

        b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", building.b_id)

    if error:
        raise RuntimeError("One or more errors, aborting")


def merge_roomfinder_rooms(data: dict[str, dict[str, Any]]) -> None:
    """
    Merge the rooms in Roomfinder with the existing data.

    This will not overwrite the existing data, but act directly on the provided data.
    """
    with (SOURCES_PATH / "16_roomfinder-merge-patches.yaml").open(encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    # It is significantly faster to first generate a lookup to the rooms in the
    # data don't need to be traversed for every single Roomfinder room.
    arch_name_lookup = {
        _data["tumonline_data"]["arch_name"].lower(): _id
        for _id, _data in data.items()
        if ("type" in _data and _data["type"] == "room" and "tumonline_data" in _data)
    }

    for room in roomfinder.Room.load_all():
        # Try to find the existing room id (which is based on the SAP Code).
        # We use the TUMonline arch_name for this, because we don't know the SAP Code here.
        try:
            r_id = _find_room_id(room, data, arch_name_lookup, patches)
            if r_id is None:
                continue
        except RoomNotFoundException as error:
            if error.known_issue:
                r_id = patches["known_issues"]["not_in_tumonline"][room.r_id]
                data[r_id] = {
                    "id": r_id,
                    "type": "room",
                    # The name might be overwritten below
                    "name": r_id if len(room.r_alias) == 0 else f"{r_id} ({room.r_alias})",
                    "parents": data[room.b_id]["parents"] + [room.b_id],
                    "data_quality": {"not_in_tumonline": True},
                }
            else:
                logging.warning(error.message)
                continue

        r_data = data[r_id]

        # TODO: Optimize integrating the alias name here
        if "(" not in r_data["name"] and len(room.r_alias) > 0:
            r_data["name"] = f"{r_data['name']} ({room.r_alias})"

        r_data["roomfinder_data"] = {
            "r_alias": room.r_alias,
            "r_number": room.r_number,
            "r_id": room.r_id,
            "r_level": room.r_level,
        }

        r_data.setdefault(
            "coords",
            {
                "lat": room.lat,
                "lon": room.lon,
                "source": "roomfinder",
            },
        )
        if room.maps:
            r_data.setdefault("maps", {})["roomfinder"] = _get_roomfinder_maps(room)

        # Add Roomfinder as source
        r_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "Roomfinder",
                "url": f"https://portal.mytum.de/displayRoomMap?roomid={room.r_id}&disable_decoration=yes",
            },
        )


def _get_roomfinder_maps(obj: roomfinder.Building | roomfinder.Room):
    """Get the maps data from a roomfinder object (room or building)"""
    # Maps metadata is extracted in another step. The data here only references the maps.
    # Maps are provided as tuples which are stored as arrays in the given JSON data.
    maps = {
        "available": [],
        "default": None,
    }
    for mapdata in obj.maps:
        maps["available"].append(
            {
                "scale": mapdata.scale,
                "id": mapdata.map_id,
                "name": mapdata.name,
                "width": mapdata.width,
                "height": mapdata.height,
            },
        )

    if not obj.default_map:
        return maps

    maps["default"] = default = obj.default_map.map_id

    # sometimes the default map is not in the available maps.
    # This is the case for example the building with id "0510"
    available_map_ids = [m["id"] for m in maps["available"]]
    if default not in available_map_ids:
        maps["available"].append(
            {
                "scale": obj.default_map.scale,
                "id": obj.default_map.map_id,
                "name": obj.default_map.name,
                "width": obj.default_map.width,
                "height": obj.default_map.height,
            },
        )
    return maps


def _find_room_id(room: roomfinder.Room, data: dict, arch_name_lookup: dict[str, str], patches) -> str | None:
    if room.r_id in patches["ignore"]:
        return None

    if room.r_id in patches["known_issues"]["mapping"]:
        return patches["known_issues"]["mapping"][room.r_id]

    if room.r_id in patches["known_issues"]["not_in_tumonline"]:
        raise RoomNotFoundException(known_issue=True)

    # Verify first, that the building is included in the data.
    # Buildings not in the data are ignored.
    if room.b_id not in data:
        return None

    search_strings = [room.r_id.lower()]
    for replacement in patches["replacements"]:
        alt_str = re.sub(replacement["search"], replacement["replace"], room.r_id)
        if alt_str != room.r_id:
            search_strings.append(alt_str.lower())

    for search in search_strings:
        if search_result := arch_name_lookup.get(search):
            return search_result

    raise RoomNotFoundException(False, f"Could not find roomfinder room in TUMonline data: {room.r_id}")


class RoomNotFoundException(Exception):
    def __init__(self, known_issue, message=None):
        self.known_issue = known_issue
        self.message = message
        super().__init__(self.message)
