import json
import logging
import re

import utm
import yaml


def merge_roomfinder_buildings(data):
    """
    Merge the buildings in Roomfinder with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """
    with open("external/buildings_roomfinder.json") as f:
        buildings = json.load(f)

    with open("sources/10_patches-roomfinder-buildings.yaml") as f:
        patches = yaml.safe_load(f.read())

    error = False
    for b in buildings:
        # 'Building' 0000 contains some buildings and places not in TUMOnline as rooms.
        # They might be integrated customly somewhere else, but here we ignore these.
        if b["b_id"] == "0000":
            continue

        for wrong, correct in patches["replacements"].items():
            if b["b_id"] == wrong:
                b["b_id"] = correct

        # Find the corresponding building in the existing data
        internal_id = None
        for _id, _data in data.items():
            if "b_prefix" in _data and _data["b_prefix"] == b["b_id"]:
                if internal_id is None:
                    internal_id = _id
                else:
                    logging.error(f"building id '{b['b_id']}' more than once in base data")
                    error = True
                    break

        if internal_id is None:
            logging.error(f"building id '{b['b_id']}' not found in base data. Add it to the areatree")
            error = True
            continue

        b_data = data[internal_id]

        b_data["roomfinder_data"] = {
            "b_id": b["b_id"],
            "b_name": b["b_name"],
            "b_alias": b["b_alias"],
            "b_area": b["b_area"],
            "b_roomCount": b["b_roomCount"],
        }

        b_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "Roomfinder",
                "url": f"https://portal.mytum.de/displayRoomMap?@{b['b_id']}",
            },
        )

        if "utm_zone" in b:
            b_data.setdefault("coords", _get_roomfinder_coords(b))
        if "maps" in b:
            b_data.setdefault("maps", {})["roomfinder"] = _get_roomfinder_maps(b)

        b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", b["b_id"])

    if error:
        raise RuntimeError("One or more errors, aborting")


def merge_roomfinder_rooms(data):
    """
    Merge the rooms in Roomfinder with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """

    with open("external/rooms_roomfinder.json") as f:
        rooms = json.load(f)

    with open("sources/16_roomfinder-merge-patches.yaml") as f:
        patches = yaml.safe_load(f.read())

    # It is significantly faster to first generate a lookup to the rooms in the
    # data don't need to be traversed for every single Roomfinder room.
    arch_name_lookup = {
        _data["tumonline_data"]["arch_name"].lower(): _id
        for _id, _data in data.items()
        if ("type" in _data and _data["type"] == "room" and "tumonline_data" in _data)
    }

    error = False
    for r in rooms:
        # Try to find the existing room id (which is based on the SAP Code).
        # We use the TUMOnline arch_name for this, because we don't know the SAP Code here.
        try:
            r_id = _find_room_id(r, data, arch_name_lookup, patches)
            if r_id is None:
                continue
        except RoomNotFoundException as e:
            if e.known_issue:
                r_id = patches["known_issues"]["not_in_tumonline"][r["r_id"]]
                data[r_id] = {
                    "id": r_id,
                    "type": "room",
                    # The name might be overwritten below
                    "name": r_id if len(r["r_alias"]) == 0 else f"{r_id} ({r['r_alias']})",
                    "parents": data[r["b_id"]]["parents"] + [r["b_id"]],
                    "data_quality": {"not_in_tumonline": True},
                }
            else:
                logging.warning(e.message)
                continue

        r_data = data[r_id]

        # TODO: Optimize integrating the alias name here
        if "(" not in r_data["name"] and len(r["r_alias"]) > 0:
            r_data["name"] = f"{r_data['name']} ({r['r_alias']})"

        r_data["roomfinder_data"] = {
            "r_alias": r["r_alias"],
            "r_number": r["r_number"],
            "r_id": r["r_id"],
            "r_level": r["r_level"],
        }

        if "utm_zone" in r:
            r_data.setdefault("coords", _get_roomfinder_coords(r))
        if "maps" in r:
            r_data.setdefault("maps", {})["roomfinder"] = _get_roomfinder_maps(r)

        # Add Roomfinder as source
        r_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "Roomfinder",
                "url": f"https://portal.mytum.de/displayRoomMap?roomid={r['r_id']}&disable_decoration=yes",
            },
        )

    if error:
        raise RuntimeError("One or more errors, aborting")


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


def _find_room_id(r, data, arch_name_lookup, patches):
    if r["r_id"] in patches["ignore"]:
        return None

    if r["r_id"] in patches["known_issues"]["mapping"]:
        return patches["known_issues"]["mapping"][r["r_id"]]

    if r["r_id"] in patches["known_issues"]["not_in_tumonline"]:
        raise RoomNotFoundException(known_issue=True)

    # Verify first, that the building is included in the data.
    # Buildings not in the data are ignored.
    if r["b_id"] not in data:
        return None

    search_strings = [r["r_id"].lower()]
    for replacement in patches["replacements"]:
        alt_str = re.sub(replacement["search"], replacement["replace"], r["r_id"])
        if alt_str != r["r_id"]:
            search_strings.append(alt_str.lower())

    for s in search_strings:
        if s in arch_name_lookup:
            return arch_name_lookup[s]

    raise RoomNotFoundException(False, f"Could not find roomfinder room in TUMOnline data: {r['r_id']}")


class RoomNotFoundException(Exception):
    def __init__(self, known_issue, message=None):
        self.known_issue = known_issue
        self.message = message
        super().__init__(self.message)
