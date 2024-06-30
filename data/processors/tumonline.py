import logging
import string
from pathlib import Path
from typing import Any

import yaml

from external.models import tumonline
from processors.merge import recursively_merge
from processors.patch import apply_roomcode_patch
from utils import TranslatableStr as _

ALLOWED_ROOMCODE_CHARS = set(string.ascii_letters) | set(string.digits) | {".", "-"}
OPERATOR_STRIP_CHARS = "[ ]"
OPERATOR_WEBNAV_LINK_PREFIX = "webnav.navigate_to?corg="

BASE = Path(__file__).parent.parent
SOURCES = BASE / "sources"


def merge_tumonline_buildings(data: dict[str, dict[str, Any]]) -> None:
    """
    Merge the buildings in TUMonline with the existing data.

    This will not overwrite the existing data, but act directly on the provided data.
    """
    error = False
    for b_id, building in tumonline.Building.load_all().items():
        # Normalize the building name (sometimes has more than one space)
        b_name = " ".join(building.name.split()).strip()

        # Extract the building id
        try:
            if int(b_id) <= 0 or int(b_id) > 9999:
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
            "filter_id": building.filter_id,
            "area_id": building.area_id,
        }

        b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", b_id)

    if error:
        raise RuntimeError("One or more errors, aborting")


# pylint: disable=too-many-locals
def merge_tumonline_rooms(data: dict[str, dict[str, Any]]) -> None:
    """
    Merge the rooms in TUMonline with the existing data.

    This will not overwrite the existing data, but act directly on the provided data.
    """
    rooms = _clean_tumonline_rooms()

    orgs_de = tumonline.Organisation.load_all_for("de")
    orgs_en = tumonline.Organisation.load_all_for("en")
    usages_lookup = tumonline.Usage.load_all()

    missing_buildings: dict[str, int] = {}
    for room_code, room in rooms.items():
        # Extract building id
        b_id = room_code.split(".")[0]
        if b_id not in data:
            missing_buildings.setdefault(b_id, 0)
            missing_buildings[b_id] += 1
            continue

        org_id_to_code = {key: org.code for key, org in orgs_de.items()}
        r_data = {
            "id": room_code,
            "type": "room",
            "name": f"{room_code} ({room.alt_name})",
            "parents": data[b_id]["parents"] + [b_id],
            "tumonline_data": {
                "tumonline_id": room.tumonline_id,
                "roomcode": room_code,
                "arch_name": room.arch_name,
                "alt_name": room.alt_name,
                "address": room.address,
                "operator": org_id_to_code.get(room.main_operator_id),
                "operator_id": room.main_operator_id,
                "operator_name": _(
                    orgs_de[room.main_operator_id].name,
                    orgs_en[room.main_operator_id].name,
                )
                if room.main_operator_id in orgs_de
                else None,
                "calendar": room.calendar_resource_nr,
                "usage": room.usage_id,
            },
            "props": {
                "ids": {
                    "roomcode": room_code,
                },
                "address": {
                    "street": room.address.street,
                    "plz_place": f"{room.address.zip_code} {room.address.place}",
                    "source": "tumonline",  # TODO: Wrong is only source is not set up to here
                },
                "stats": {
                    "n_seats_sitting": room.seats.sitting,
                    "n_seats_standing": room.seats.standing,
                    "n_seats_wheelchair": room.seats.wheelchair,
                },
            },
        }

        if room.arch_name:
            r_data["props"]["ids"]["arch_name"] = room.arch_name
        room.seats.model_dump()
        # Usage
        if room.usage_id in usages_lookup:
            tumonline_usage = usages_lookup[room.usage_id]
            r_data["usage"] = {
                "name": _(tumonline_usage.name),
                "din_277": tumonline_usage.din277_id,
                "din_277_desc": tumonline_usage.din277_name,
            }
        else:
            logging.error(f"Unknown usage for room '{room['roomcode']}': Id '{room['usage']}'")
            continue

        # TUMonline data does not overwrite the existing data when merged
        recursively_merge(data, {r_data["id"]: r_data}, overwrite=False)

        # Add TUMonline as source
        data[r_data["id"]].setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "TUMonline",
                "url": f"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/{room.tumonline_id}",
            },
        )
        if room.patched:
            data[r_data["id"]]["sources"]["patched"] = True

    if parentless := [(b_id, content) for b_id, content in data.items() if "parents" not in content]:
        for b_id, content in parentless:
            logging.critical(f"No parents exist for {b_id}: {content}")
        logging.critical("This is probably the case, because roompatches were renamed upstream")
        raise RuntimeError("Invariant not preserved")
    if missing_buildings:
        logging.warning(
            f"Ignored {sum(missing_buildings.values())} rooms for the following buildings, "
            f"which were not found: {sorted(missing_buildings.keys())}",
        )


def _clean_tumonline_rooms():
    """
    Apply some known corrections / patches on the TUMonline room data.

    It also searches for inconsistencies not yet patched
    """
    rooms = tumonline.Room.load_all()

    with open(SOURCES / "15_patches-rooms_tumonline.yaml", encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    apply_roomcode_patch(rooms, patches["patches"])

    used_arch_names: dict[str, tuple[str, str, str]] = {}
    used_roomcode_levels = {}
    invalid_rooms: list[str] = []
    for room_code, room in rooms.items():
        if not room.arch_name or not room.alt_name:
            logging.warning(f"ignoring {room_code} as it has {room.arch_name=}, {room.alt_name=}")
            invalid_rooms.append(room_code)
            continue
        # Validate the room_code
        roomcode_parts: tuple[str, str, str] = room_code.split(".")
        if len(roomcode_parts) != 3:
            logging.warning(f"Invalid roomcode: Not three '.'-separated parts: {room['roomcode']}")
            invalid_rooms.append(room_code)
        if len(set(room_code) - ALLOWED_ROOMCODE_CHARS) > 0:
            logging.warning(
                f"Invalid character(s) in roomcode '{room['roomcode']}': "
                f"{set(room['roomcode']) - ALLOWED_ROOMCODE_CHARS}",
            )
            invalid_rooms.append(room_code)

        if roomcode_parts[1] not in used_roomcode_levels:
            used_roomcode_levels[roomcode_parts[1]] = room_code

        # Validate the arch_name.
        arch_name_parts: tuple[str, str] = room.arch_name.split("@")
        if len(arch_name_parts) != 2:
            logging.warning(f"Invalid arch_name: No '@' in '{room['arch_name']}' (room {room['roomcode']})")
            invalid_rooms.append(room_code)
        if len(arch_name_parts[1]) != 4 or not arch_name_parts[1].isdigit():
            logging.warning(
                f"Invalid building specification in arch_name: Not four digits: "
                f"'{arch_name_parts[1]}' in '{room['arch_name']}' (room {room['roomcode']})",
            )
            invalid_rooms.append(room_code)

        _infer_arch_name(room, arch_name_parts, used_arch_names, roomcode_parts, rooms)

    if invalid_rooms:
        logging.warning(f"Ignored {len(invalid_rooms)} TUMonline rooms because they are invalid.")
        for room_code in invalid_rooms:
            rooms.pop(room_code)

    return rooms


def _infer_arch_name(
    room: tumonline.Room,
    arch_name_parts: tuple[str, str],
    used_arch_names: dict[str, tuple[str, str, str]],
    roomcode_parts: tuple[str, str, str],
    rooms: dict[str, tumonline.Room],
) -> None:
    """Infer the arch name and other related properties"""
    # Some rooms don't have an arch_name. The value is then usually just like "@1234".
    # Since this is not helpful (the building is already known for all rooms) rooms are then dropped.
    if len(arch_name_parts[0]) == 0:
        room.arch_name = ""
        return

    # THIS SECTION MIGHT CHANGE THE ARCH_NAME as well
    _maybe_set_alt_name(".".join(roomcode_parts), arch_name_parts, room)

    # Check for arch_name and roomcode suffix mismatch:
    if (
        arch_name_parts[0][-1].isalpha()
        and roomcode_parts[2][-1].isalpha()
        and arch_name_parts[0][-1].lower() != roomcode_parts[2][-1].lower()
    ):
        # For the MW buildings it seems that there is a difference between
        # upper- and lowercase suffixes which is probably not representable in the TUMonline system.
        # That is why lowercase suffixes in the arch_name might be different from the suffix in the roomcode
        if roomcode_parts[0].startswith("550"):
            pass
        else:
            # TODO: This code section might need to be continued
            # logging.debug(f"{r["arch_name"]=}, {roomcode_parts[2]=}")
            pass

    # Check for duplicate uses of arch names
    if room.arch_name not in used_arch_names:
        used_arch_names[room.arch_name] = roomcode_parts
        return

    r1_parts = roomcode_parts
    r2_parts = used_arch_names[room.arch_name]
    if r1_parts[0] == r2_parts[0] and r1_parts[2] == r2_parts[2]:
        return

    a_name = arch_name_parts[0].lower()
    # Commonly: "-1405@0504" is arch_name for both "0504.U1.405" and "0504.U1.405A"
    # In this case: add the suffix to the arch_name for the second room
    if (a_name.endswith(r1_parts[2].lower()) and not a_name.endswith(r2_parts[2].lower())) or (
        a_name.endswith(r2_parts[2].lower()) and not a_name.endswith(r1_parts[2].lower())
    ):
        return
    # Sometimes the arch_name matches only one of the two roomcodes
    min_len = min(len(r1_parts[2]), len(r2_parts[2]))
    if r1_parts[2][:min_len] == r2_parts[2][:min_len]:
        if len(r1_parts[2]) > len(r2_parts[2]):
            room.arch_name = arch_name_parts[0] + r1_parts[2][min_len:].lower() + "@" + arch_name_parts[1]
        else:
            looked_up_r2 = rooms[".".join(used_arch_names[room.arch_name])]
            looked_up_r2["arch_name"] = arch_name_parts[0] + r2_parts[2][min_len:].lower() + "@" + arch_name_parts[1]
            used_arch_names[room.arch_name] = roomcode_parts
        room.patched = True


def _maybe_set_alt_name(room_code: str, arch_name_parts: tuple[str, str], room: tumonline.Room) -> None:
    """
    Deduces the alt_name from the roomname

    The alt_name commonly begins with the roomname.
    Since ther roomname should be encoded in the arch_name as the part before the "@" we verify,
    that these match and remove the roomname in the alt_name (which is more like a descriptive name).
    As far as we observed so far, if the room has no arch_name it also doesn't have any roomname in the alt_name.
    Also, if there are no comma-separated parts, the roomname is usually not in the alt_name.
    """
    alt_parts = room.alt_name.split(",")
    if len(alt_parts) < 2:
        return
    if alt_parts[0].lower() == arch_name_parts[0].lower():
        room.alt_name = ", ".join(alt_parts[1:])
        return
    # The most common mismatch is if the roomname in the alt_name is like "L516" and the arch_name starts with "L 516".
    # In this case we change the arch_name to the format without a space
    joined_roomname = arch_name_parts[0].replace(" ", "", 2)
    if arch_name_parts[0][:2] in {"R ", "L ", "M ", "N "} and alt_parts[0] == joined_roomname:
        arch_name_parts = alt_parts[0], arch_name_parts[1]
        room.arch_name = "@".join(arch_name_parts)
        room.alt_name = ", ".join(alt_parts[1:])
        room.patched = True
    # The same might appear the other way round (e.g. "N 1070 ZG" and "N1070ZG")
    elif alt_parts[0][:2] in {"N ", "R "} and arch_name_parts[0] == alt_parts[0].replace(" ", ""):
        room.alt_name = ", ".join(alt_parts[1:])
        room.patched = True
    # The second most common mismatch is if the roomname in the alt_name is prepended with the abbrev of the building
    # Example: "MW 1050"
    elif any(alt_parts[0].startswith(s) for s in ["PH ", "MW ", "WSI ", "CH ", "MI "]):
        room.alt_name = ", ".join(alt_parts[1:])
        room.patched = True
    # If the roomname has a comma, the comparison by parts fails
    elif "," in arch_name_parts[0] and room.alt_name.startswith(arch_name_parts[0]):
        alt_name_index = arch_name_parts[0].count(",") + 1
        room.alt_name = ", ".join(alt_parts[alt_name_index:])
        room.patched = True
    # The Theresianum is an exception where the roomname is the second part of the alt_name.
    # Both are discarded since both roomcode and building name are given separately
    elif alt_parts[0] == "Theresianum" and alt_parts[1] == arch_name_parts[0]:
        room.alt_name = ", ".join(alt_parts[2:])
        room.patched = True
    else:
        logging.debug(
            f"(alt_name / arch_name mismatch): {alt_parts[0]=} {arch_name_parts[0]=} {room_code=}",
        )
    room.alt_name = room.alt_name.strip()
