import json
import logging
import re

import utm  # type: ignore
import yaml


def merge_roomfinder_buildings(data):
    """
    Merge the buildings in Roomfinder with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """
    with open("external/buildings_roomfinder.json", encoding="utf-8") as file:
        buildings = json.load(file)

    with open("sources/10_patches-roomfinder-buildings.yaml", encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    error = False
    for building in buildings:
        # 'Building' 0000 contains some buildings and places not in TUMOnline as rooms.
        # They might be integrated customly somewhere else, but here we ignore these.
        if building["b_id"] == "0000":
            continue

        for wrong, correct in patches["replacements"].items():
            if building["b_id"] == wrong:
                building["b_id"] = correct

        # Find the corresponding building in the existing data
        internal_id = None
        for _id, _data in data.items():
            if "b_prefix" in _data and _data["b_prefix"] == building["b_id"]:
                if internal_id is None:
                    internal_id = _id
                else:
                    logging.error(f"building id '{building['b_id']}' more than once in base data")
                    error = True
                    break

        if internal_id is None:
            logging.error(f"building id '{building['b_id']}' not found in base data. Add it to the areatree")
            error = True
            continue

        b_data = data[internal_id]

        b_data["roomfinder_data"] = {
            "b_id": building["b_id"],
            "b_name": building["b_name"],
            "b_alias": building["b_alias"],
            "b_area": building["b_area"],
            "b_roomCount": building["b_roomCount"],
        }

        b_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "Roomfinder",
                "url": f"https://portal.mytum.de/displayRoomMap?@{building['b_id']}",
            },
        )

        if "utm_zone" in building:
            b_data.setdefault("coords", _get_roomfinder_coords(building))
        if "maps" in building:
            b_data.setdefault("maps", {})["roomfinder"] = _get_roomfinder_maps(building)

        b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", building["b_id"])

    if error:
        raise RuntimeError("One or more errors, aborting")


def merge_roomfinder_rooms(data):
    """
    Merge the rooms in Roomfinder with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """

    with open("external/rooms_roomfinder.json", encoding="utf-8") as file:
        rooms = json.load(file)

    with open("sources/16_roomfinder-merge-patches.yaml", encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    # It is significantly faster to first generate a lookup to the rooms in the
    # data don't need to be traversed for every single Roomfinder room.
    arch_name_lookup = {
        _data["tumonline_data"]["arch_name"].lower(): _id
        for _id, _data in data.items()
        if ("type" in _data and _data["type"] == "room" and "tumonline_data" in _data)
    }

    for room in rooms:
        # Try to find the existing room id (which is based on the SAP Code).
        # We use the TUMOnline arch_name for this, because we don't know the SAP Code here.
        try:
            r_id = _find_room_id(room, data, arch_name_lookup, patches)
            if r_id is None:
                continue
        except RoomNotFoundException as error:
            if error.known_issue:
                r_id = patches["known_issues"]["not_in_tumonline"][room["r_id"]]
                data[r_id] = {
                    "id": r_id,
                    "type": "room",
                    # The name might be overwritten below
                    "name": r_id if len(room["r_alias"]) == 0 else f"{r_id} ({room['r_alias']})",
                    "parents": data[room["b_id"]]["parents"] + [room["b_id"]],
                    "data_quality": {"not_in_tumonline": True},
                }
            else:
                logging.warning(error.message)
                continue

        r_data = data[r_id]

        # TODO: Optimize integrating the alias name here
        if "(" not in r_data["name"] and len(room["r_alias"]) > 0:
            r_data["name"] = f"{r_data['name']} ({room['r_alias']})"

        r_data["roomfinder_data"] = {
            "r_alias": room["r_alias"],
            "r_number": room["r_number"],
            "r_id": room["r_id"],
            "r_level": room["r_level"],
        }

        if "utm_zone" in room:
            r_data.setdefault("coords", _get_roomfinder_coords(room))
        if "maps" in room:
            r_data.setdefault("maps", {})["roomfinder"] = _get_roomfinder_maps(room)

        # Add Roomfinder as source
        r_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "Roomfinder",
                "url": f"https://portal.mytum.de/displayRoomMap?roomid={room['r_id']}&disable_decoration=yes",
            },
        )


def _get_roomfinder_coords(obj):
    """Get the coordinates from a roomfinder object (room or building)"""
    # UTM zone is either "32" or "33", corresponding to zones "32U" and "33U"
    # TODO: Map image boundaries also included "33T". It could maybe be possible to guess
    #       whether it is "U" or "T" based on the northing (which is always the distance
    #       to the equator).
    if obj["utm_zone"] not in {"32", "33"}:
        logging.error(f"Unexpected UTM zone '{obj['utm_zone']}'")
    lat, lon = utm.to_latlon(obj["utm_easting"], obj["utm_northing"], int(obj["utm_zone"]), "U")

    return {
        "lat": lat,
        "lon": lon,
        "utm": {
            "zone_number": int(obj["utm_zone"]),
            "zone_letter": "U",
            "easting": obj["utm_easting"],
            "northing": obj["utm_northing"],
        },
        "source": "roomfinder",
    }


def _get_roomfinder_maps(obj):
    """Get the maps data from a roomfinder object (room or building)"""
    # Maps metadata is extracted in another step. The data here only references the maps.
    # Maps are provided as tuples which are stored as arrays in the given JSON data.
    maps = {
        "available": [],
        "default": None,
    }
    if len(obj["maps"]) > 0:
        for mapdata in obj["maps"]:
            maps["available"].append(
                {
                    "id": f"rf{mapdata[1]}",  # Roomfinder data is with ints as id, but we use a string based format
                    "scale": mapdata[0],
                    "name": mapdata[2],
                    "width": mapdata[3],
                    "height": mapdata[4],
                },
            )

    if not obj["default_map"]:
        return maps

    # If the default map is the world map, this is usually
    # the only map available. As we don't include the world
    # map into the available maps, return empty data here
    if obj["default_map"][1] == 9:
        maps["available"].clear()
        return maps

    maps["default"] = default = f"rf{obj['default_map'][1]}"

    # sometimes the default map is not in the available maps.
    # This is the case for example the building with id "0510"
    available_map_ids = [m["id"] for m in maps["available"]]
    if default and default not in available_map_ids:
        mapdata = obj["default_map"]
        maps["available"].append(
            {
                "id": f"rf{mapdata[1]}",  # Roomfinder data is with ints as id, but we use a string based format
                "scale": mapdata[0],
                "name": mapdata[2],
                "width": mapdata[3],
                "height": mapdata[4],
            },
        )
    return maps


def _find_room_id(room, data, arch_name_lookup, patches):
    if room["r_id"] in patches["ignore"]:
        return None

    if room["r_id"] in patches["known_issues"]["mapping"]:
        return patches["known_issues"]["mapping"][room["r_id"]]

    if room["r_id"] in patches["known_issues"]["not_in_tumonline"]:
        raise RoomNotFoundException(known_issue=True)

    # Verify first, that the building is included in the data.
    # Buildings not in the data are ignored.
    if room["b_id"] not in data:
        return None

    search_strings = [room["r_id"].lower()]
    for replacement in patches["replacements"]:
        alt_str = re.sub(replacement["search"], replacement["replace"], room["r_id"])
        if alt_str != room["r_id"]:
            search_strings.append(alt_str.lower())

    for search in search_strings:
        if search in arch_name_lookup:
            return arch_name_lookup[search]

    raise RoomNotFoundException(False, f"Could not find roomfinder room in TUMOnline data: {room['r_id']}")


class RoomNotFoundException(Exception):
    def __init__(self, known_issue, message=None):
        self.known_issue = known_issue
        self.message = message
        super().__init__(self.message)
