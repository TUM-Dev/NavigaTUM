import json
import logging
import string

import yaml
from processors.merge import merge_object
from processors.patch import apply_patches

ALLOWED_ROOMCODE_CHARS = set(string.ascii_letters) | set(string.digits) | {".", "-"}


def merge_tumonline_buildings(data):
    """
    Merge the buildings in TUMOnline with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """
    with open("external/buildings_tumonline.json", encoding="utf-8") as file:
        buildings = json.load(file)

    error = False
    for building in buildings:
        # Normalize the building name (sometimes has more than one space)
        b_name = " ".join(building["name"].split()).strip()

        # Extract the building id
        try:
            b_id = b_name.split(" ", 2)[0]
            if 0 >= int(b_id) or int(b_id) > 9999:
                logging.error(f"Invalid building id '{b_id}' for building '{b_name}', expected it to be in 1..9999")
                error = True
                continue
        except ValueError:
            error = True
            logging.error(f"Failed to parse building name as '1234 [...]' with a number in 1..9999 for: '{b_name}'")
            continue

        # Find the corresponding building in the existing data
        internal_id = None
        for _id, _data in data.items():
            if "b_prefix" in _data and _data["b_prefix"] == b_id:
                if internal_id is None:
                    internal_id = _id
                else:
                    logging.warning(f"building id '{b_id}' ('{b_name}') more than once in base data")
                    error = True
                    break

        if internal_id is None:
            # Currently, not an error, because the areatree is built by hand.
            # This is just to show warn these buildings are not included there.
            logging.warning(f"building id '{b_id}' ('{b_name}') not found in base data, ignoring")
            continue

        b_data = data[internal_id]

        # The address is not extracted from the building name here.
        # It might later be inferred from the more detailed room information
        # of a room in this building.

        b_data["tumonline_data"] = {
            "name": b_name,
            "filter_id": building["filter_id"],
            "area_id": building["area_id"],
        }

        b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", b_id)

    if error:
        raise RuntimeError("One or more errors, aborting")
    return data


def merge_tumonline_rooms(data):
    """
    Merge the rooms in TUMOnline with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """
    with open("external/rooms_tumonline.json", encoding="utf-8") as file:
        rooms = json.load(file)

    with open("external/usages_tumonline.json", encoding="utf-8") as file:
        usages = json.load(file)
    usages_lookup = {usage["id"]: usage for usage in usages}

    rooms = _clean_tumonline_rooms(rooms)

    missing_buildings = {}
    for room in rooms:
        # Extract building id
        b_id = room["roomcode"].split(".")[0]
        if b_id not in data:
            missing_buildings.setdefault(b_id, 0)
            missing_buildings[b_id] += 1
            continue

        r_data = {
            "id": room["roomcode"],
            "type": "room",
            "name": f"{room['roomcode']} ({room['alt_name']})",
            "parents": data[b_id]["parents"] + [b_id],
            "tumonline_data": {
                "roomcode": room["roomcode"],
                "arch_name": room["arch_name"],
                "alt_name": room["alt_name"],
                "address": _clean_spaces(room["address"]),
                "address_link": room["address_link"],
                "plz_place": room["plz_place"],
                "operator": room["operator"],
                "operator_link": room["op_link"],
                "room_link": room["room_link"],
                "calendar": room["calendar"],
                "b_filter_id": room["b_filter_id"],
                "b_area_id": room["b_area_id"],
                "usage": room["usage"],
            },
            "props": {
                "ids": {
                    "roomcode": room["roomcode"],
                },
                "address": {
                    "street": _clean_spaces(room["address"]),
                    "plz_place": room["plz_place"],
                    "source": "tumonline",  # TODO: Wrong is only source is not set up to here
                },
            },
        }

        if len(room["arch_name"]) > 0:
            r_data["props"]["ids"]["arch_name"] = room["arch_name"]

        if "extended" in room and "physikalische Eigenschaften" in room["extended"]:
            physical_props = room["extended"]["physikalische Eigenschaften"]
            r_data["props"]["stats"] = {}
            if "Sitzplätze" in physical_props:
                r_data["props"]["stats"]["n_seats"] = int(physical_props["Sitzplätze"])

        # Usage
        if room["usage"] in usages_lookup:
            tumonline_usage = usages_lookup[room["usage"]]
            parts = tumonline_usage["din_277"].split(" - ")
            r_data["usage"] = {
                "name": tumonline_usage["name"],
                "din_277": parts[0],
                "din_277_desc": parts[1],
            }
        else:
            logging.error(f"Unknown usage for room '{room['roomcode']}': Id '{room['usage']}'")
            continue

        if "extended" in room:
            r_data["tumonline_data"]["extended"] = room["extended"]

        # TUMOnline data does not overwrite the existing data when merged
        merge_object(data, {r_data["id"]: r_data}, overwrite=False)

        # Add TUMOnline as source
        data[r_data["id"]].setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "TUMOnline",
                "url": f"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/{room['room_link']}",
            },
        )
        if room["patched"]:
            data[r_data["id"]]["sources"]["patched"] = True

    if len(missing_buildings) > 0:
        logging.warning(
            f"Ignored {sum(missing_buildings.values())} rooms for the following buildings, "
            f"which were not found: {sorted(missing_buildings.keys())}",
        )


def _clean_tumonline_rooms(to_rooms):
    """
    This applies some known corrections / patches on the TUMOnline room data.
    It also searches for inconsitencies not yet patched
    """

    roomcode_lookup = {r["roomcode"]: r for r in to_rooms}

    with open("sources/15_patches-rooms_tumonline.yaml", encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    patched_rooms = apply_patches(to_rooms, patches["patches"], "roomcode")
    patched_room_ids = {r["roomcode"] for r in patched_rooms}

    used_arch_names = {}
    used_roomcode_levels = {}
    invalid_rooms = []
    for room in to_rooms:
        # Keep track of whether changes were made
        room.setdefault("patched", False)

        if room["roomcode"] in patched_room_ids:
            room["patched"] = True

        # Validate the roomcode
        roomcode_parts = room["roomcode"].split(".")
        if len(roomcode_parts) != 3:
            logging.warning(f"Invalid roomcode: Not three '.'-separated parts: {room['roomcode']}")
            invalid_rooms.append(room)
        if len(set(room["roomcode"]) - ALLOWED_ROOMCODE_CHARS) > 0:
            logging.warning(
                f"Invalid character(s) in roomcode '{room['roomcode']}': "
                f"{set(room['roomcode']) - ALLOWED_ROOMCODE_CHARS}",
            )
            invalid_rooms.append(room)

        if roomcode_parts[1] not in used_roomcode_levels:
            used_roomcode_levels[roomcode_parts[1]] = room["roomcode"]

        # Validate the arch_name.
        arch_name_parts = room["arch_name"].split("@")
        if len(arch_name_parts) != 2:
            logging.warning(f"Invalid arch_name: No '@' in '{room['arch_name']}' (room {room['roomcode']})")
            invalid_rooms.append(room)
        if len(arch_name_parts[1]) != 4 or not arch_name_parts[1].isdigit():
            logging.warning(
                f"Invalid building specification in arch_name: Not four digits: "
                f"'{arch_name_parts[1]}' in '{room['arch_name']}' (room {room['roomcode']})",
            )
            invalid_rooms.append(room)

        _infer_arch_name(room, arch_name_parts, used_arch_names, roomcode_parts, roomcode_lookup)

        # The address commonly has duplicate spaces
        room["address"] = _clean_spaces(room["address"])
    if len(invalid_rooms) > 0:
        for room in invalid_rooms:
            to_rooms.remove(room)

        logging.warning(f"Ignored {len(invalid_rooms)} TUMOnline rooms because they are invalid.")

    return to_rooms


def _infer_arch_name(room, arch_name_parts, used_arch_names, roomcode_parts, roomcode_lookup):
    """
    Infer the arch name and other related properties
    """

    # Some rooms don't have an arch_name. The value is then usually just like "@1234".
    # Since this is not helpful (the building is already known for all rooms) rooms are then dropped.
    if len(arch_name_parts[0]) == 0:
        room["arch_name"] = ""
        return

    # The alt_name commonly begins with the roomname. Since ther roomname should be
    # encoded in the arch_name as the part before the "@" we verify, that these match
    # and remove the roomname in the alt_name (which is more like a descriptive name).
    # As far as I observed so far, if the room has no arch_name it also doesn't have
    # any roomname in the alt_name. Also, if there are no comma-separated parts, the
    # roomname is usually not in the alt_name.
    # THIS SECTION MIGHT CHANGE THE ARCH_NAME
    alt_parts = [_clean_spaces(s) for s in room["alt_name"].split(",")]
    if len(alt_parts) >= 2:
        if alt_parts[0].lower() == arch_name_parts[0].lower():
            room["alt_name"] = ", ".join(alt_parts[1:])
        else:
            # The most common mismatch is if the roomname in the alt_name is like "L516"
            # and the arch_name starts with "L 516". In this case we change the arch_name
            # to the format without a space
            joined_roomname = arch_name_parts[0].replace(" ", "", 2)
            if arch_name_parts[0][:2] in {"R ", "L ", "M ", "N "} and alt_parts[0] == joined_roomname:
                room["alt_roomname"] = arch_name_parts[0]
                arch_name_parts[0] = alt_parts[0]
                room["arch_name"] = "@".join(arch_name_parts)
                room["alt_name"] = ", ".join(alt_parts[1:])
                room["patched"] = True
            # The same might appear the other way round (e.g. "N 1070 ZG" and "N1070ZG")
            elif alt_parts[0][:2] in {"N ", "R "} and arch_name_parts[0] == alt_parts[0].replace(" ", ""):
                room["alt_roomname"] = alt_parts[0]
                room["alt_name"] = ", ".join(alt_parts[1:])
                room["patched"] = True
            # The second most common mismatch is if the roomname in the alt_name is prepended
            # with the abbrev of the building like "MW 1050"
            elif any(alt_parts[0].startswith(s) for s in ["PH ", "MW ", "WSI ", "CH ", "MI "]):
                room["alt_name"] = ", ".join(alt_parts[1:])
                room["patched"] = True
            # If the roomname has a comma, the comparision by parts fails
            elif "," in arch_name_parts[0] and room["alt_name"].startswith(arch_name_parts[0]):
                alt_name_index = arch_name_parts[0].count(",") + 1
                room["alt_name"] = ", ".join(alt_parts[alt_name_index:])
                room["patched"] = True
            # The Theresianum is an exception where the roomname is the second part of the
            # alt_name. Both are discarded since both roomcode and building name are given
            # separately
            elif alt_parts[0] == "Theresianum" and alt_parts[1] == arch_name_parts[0]:
                room["alt_name"] = ", ".join(alt_parts[2:])
                room["patched"] = True
            else:
                logging.debug(
                    f"(alt_name / arch_name mismatch): " f"{alt_parts[0]=} {arch_name_parts[0]=} {room['roomcode']=}",
                )

    # Check for arch_name and roomcode suffix mismatch:
    if (
            arch_name_parts[0][-1].isalpha()
            and roomcode_parts[2][-1].isalpha()
            and arch_name_parts[0][-1].lower() != roomcode_parts[2][-1].lower()
    ):
        # For the MW buildings it seems that there is a difference between
        # upper- and lowercase suffixes which is probably not representable in the TUMOnline system.
        # That is why lowercase suffixes in the arch_name might be different from the suffix in the roomcode
        if roomcode_parts[0].startswith("550"):
            pass
        else:
            # TODO: This code section might need to be continued
            # logging.debug(f"{r["arch_name"]=}, {roomcode_parts[2]=}")
            pass

    # Check for duplicate uses of arch names
    if room["arch_name"] not in used_arch_names:
        used_arch_names[room["arch_name"]] = room["roomcode"]
        return

    r1_parts = roomcode_parts
    r2_parts = used_arch_names[room["arch_name"]].split(".")
    if r1_parts[0] == r2_parts[0] and r1_parts[2] == r2_parts[2]:
        return

    a_name = arch_name_parts[0].lower()
    # Commonly: "-1405@0504" is arch_name for both "0504.U1.405" and "0504.U1.405A"
    # In this case: add the suffix to the arch_name for the second room
    if (
        (a_name.endswith(r1_parts[2].lower()) and not a_name.endswith(r2_parts[2].lower())) or
        (a_name.endswith(r2_parts[2].lower()) and not a_name.endswith(r1_parts[2].lower()))
    ):
        return
    # Sometimes the arch_name matches only one of the two roomcodes
    min_len = min(len(r1_parts[2]), len(r2_parts[2]))
    if r1_parts[2][:min_len] == r2_parts[2][:min_len]:
        if len(r1_parts[2]) > len(r2_parts[2]):
            room["arch_name"] = arch_name_parts[0] + r1_parts[2][min_len:].lower() + "@" + arch_name_parts[1]
        else:
            r2 = roomcode_lookup[used_arch_names[room["arch_name"]]]
            r2["arch_name"] = arch_name_parts[0] + r2_parts[2][min_len:].lower() + "@" + arch_name_parts[1]
            used_arch_names[room["arch_name"]] = room["roomcode"]
        room["patched"] = True


def _clean_spaces(_string: str) -> str:
    """Remove leading and trailing spaces as well as duplicate spaces in-between"""
    return " ".join(_string.split())
