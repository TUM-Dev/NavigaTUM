import orjson
import logging
import string
from pathlib import Path
from typing import Any

import polars as pl
import yaml

from external.loaders.tumonline import load_buildings, load_orgs, load_usages
from external.models import tumonline
from processors.df_utils import ensure_column, ensure_columns, to_json_or_none, translatable_to_columns
from processors.patch import apply_roomcode_patch
from utils import TranslatableStr as _

ALLOWED_ROOMCODE_CHARS = set(string.ascii_letters) | set(string.digits) | {".", "-"}
OPERATOR_STRIP_CHARS = "[ ]"
OPERATOR_WEBNAV_LINK_PREFIX = "webnav.navigate_to?corg="

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"


def merge_tumonline_buildings(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge the buildings in TUMonline with the existing data.

    Returns a new DataFrame with tumonline building data merged in.
    """
    error = False
    buildings_rows: list[dict[str, Any]] = []
    for building in load_buildings().iter_rows(named=True):
        b_id = building["building_key"]
        b_name = " ".join(building["name"].split()).strip()

        # Check for duplicates in the DataFrame
        matches = df.filter(pl.col("b_prefix") == b_id)
        if matches.height > 1:
            logging.warning(f"building id '{b_id}' ('{b_name}') more than once in base data")
            error = True
            continue
        if matches.height == 0:
            # Currently, not an error, because the areatree is built by hand.
            logging.warning(f"building id '{b_id}' ('{b_name}') not found in base data, ignoring")
            continue

        buildings_rows.append(
            {
                "b_prefix_match": b_id,
                "tumonline_data_json_new": orjson.dumps(
                    {"name": b_name, "filter_id": building["filter_id"], "area_id": building["area_id"]},
                ).decode(),
                "props_ids_b_id_new": b_id,
            }
        )

    if error:
        raise RuntimeError("One or more errors, aborting")

    if not buildings_rows:
        return df

    buildings_df = pl.DataFrame(buildings_rows)

    # Ensure columns exist before coalescing
    for col in ["tumonline_data_json", "props_ids_b_id"]:
        df = ensure_column(df, col)

    # Join on b_prefix
    result = df.join(buildings_df, left_on="b_prefix", right_on="b_prefix_match", how="left")

    # Merge: tumonline_data_json gets overwritten, props_ids_b_id uses setdefault (don't overwrite)
    result = result.with_columns(
        [
            pl.coalesce(pl.col("tumonline_data_json_new"), pl.col("tumonline_data_json")).alias("tumonline_data_json"),
            pl.coalesce(pl.col("props_ids_b_id"), pl.col("props_ids_b_id_new")).alias("props_ids_b_id"),
        ]
    )
    result = result.drop(["tumonline_data_json_new", "props_ids_b_id_new"])

    return result


_BUILDING_TYPES = pl.Series(["building", "joined_building"])


def merge_tumonline_rooms(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge the rooms in TUMonline with the existing data.

    Returns a new DataFrame with tumonline room data merged in.
    """
    rooms = _clean_tumonline_rooms()

    orgs_lookup = {
        row["org_id"]: row
        for row in load_orgs("de")
        .select("org_id", "code", pl.col("name").alias("name_de"))
        .join(
            load_orgs("en").select("org_id", pl.col("name").alias("name_en")),
            on="org_id",
            how="inner",
        )
        .iter_rows(named=True)
    }
    usages_lookup = {row["usage_id"]: row for row in load_usages().iter_rows(named=True)}

    building_parents = {
        brow["id"]: (brow["parents"] or [])
        for brow in df.filter(pl.col("type").is_in(_BUILDING_TYPES)).select("id", "parents").iter_rows(named=True)
    }

    candidate_rows: list[dict[str, Any]] = []
    missing_buildings: dict[str, int] = {}

    for room_code, room in rooms.items():
        b_id = room_code.split(".")[0]
        if b_id not in building_parents:
            missing_buildings[b_id] = missing_buildings.get(b_id, 0) + 1
            continue

        if room.usage_id not in usages_lookup:
            logging.error(f"Unknown usage for room '{room_code}': Id '{room.usage_id}'")
            continue
        tumonline_usage = usages_lookup[room.usage_id]
        usage_name = _(tumonline_usage["name"])

        op = orgs_lookup.get(room.main_operator_id)
        operator_name = _(op["name_de"], op["name_en"]) if op else None

        tumonline_data = {
            "tumonline_id": room.tumonline_id,
            "roomcode": room_code,
            "arch_name": room.arch_name,
            "alt_name": room.alt_name,
            "address": {"place": room.address.place, "street": room.address.street, "zip_code": room.address.zip_code},
            "operator": op["code"] if op else None,
            "operator_id": room.main_operator_id,
            "operator_name": operator_name,
            "calendar": room.calendar_resource_nr,
            "usage": room.usage_id,
        }
        source_entry = {
            "name": "TUMonline",
            "url": f"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/wbRaum.editRaum?pRaumNr={room.tumonline_id}",
        }
        candidate_rows.append(
            {
                "id": room_code,
                "type": "room",
                "name": f"{room_code} ({room.alt_name})",
                "name_de": f"{room_code} ({room.alt_name})",
                "name_en": f"{room_code} ({room.alt_name})",
                "parents": building_parents[b_id] + [b_id],
                "tumonline_data_json": to_json_or_none(tumonline_data),
                "props_ids_roomcode": room_code,
                "props_ids_arch_name": room.arch_name if room.arch_name else None,
                "props_address_street": room.address.street,
                "props_address_plz_place": f"{room.address.zip_code} {room.address.place}",
                "props_address_source": "tumonline",
                "props_stats_n_seats_sitting": room.seats.sitting,
                "props_stats_n_seats_standing": room.seats.standing,
                "props_stats_n_seats_wheelchair": room.seats.wheelchair,
                **translatable_to_columns("usage_name", usage_name),
                "usage_din_277": tumonline_usage["din277_id"],
                "usage_din_277_desc": tumonline_usage["din277_name"],
                "sources_base_json": to_json_or_none([source_entry]),
                "sources_patched": True if room.patched else None,
            }
        )

    if missing_buildings:
        logging.warning(
            f"Ignored {sum(missing_buildings.values())} rooms for the following buildings, "
            f"which were not found: {sorted(missing_buildings.keys())}",
        )

    if candidate_rows:
        rooms_df = pl.DataFrame(candidate_rows, infer_schema_length=None)
        if rooms_df.schema.get("sources_patched") == pl.Null:
            rooms_df = rooms_df.with_columns(pl.col("sources_patched").cast(pl.Boolean))

        existing_ids = df.select("id")
        new_rooms = rooms_df.join(existing_ids, on="id", how="anti")
        update_rooms = rooms_df.join(existing_ids, on="id", how="semi")

        if new_rooms.height > 0:
            df = pl.concat([df, new_rooms], how="diagonal_relaxed")

        if update_rooms.height > 0:
            update_cols = [c for c in update_rooms.columns if c != "id"]
            df = ensure_columns(df, {c: update_rooms.schema.get(c, pl.Utf8()) for c in update_cols})
            df = df.join(update_rooms, on="id", how="left", suffix="_update")

            # Existing value wins (setdefault semantics)
            df = df.with_columns(
                [
                    pl.coalesce(pl.col(c), pl.col(f"{c}_update")).alias(c)
                    for c in update_cols
                    if f"{c}_update" in df.columns
                ]
            )

            # Append TUMonline source string and OR in the patched flag
            df = ensure_column(df, "sources_patched", pl.Boolean())
            new_src = pl.col("sources_base_json_update")
            df = df.with_columns(
                pl.when(new_src.is_not_null() & pl.col("sources_base_json").is_not_null())
                .then(pl.col("sources_base_json").str.strip_suffix("]") + "," + new_src.str.strip_prefix("["))
                .when(new_src.is_not_null())
                .then(new_src)
                .otherwise(pl.col("sources_base_json"))
                .alias("sources_base_json"),
                pl.when(pl.col("sources_patched_update") == True)  # noqa: E712
                .then(pl.lit(True))
                .otherwise(pl.col("sources_patched"))
                .alias("sources_patched"),
            )
            df = df.drop([c for c in df.columns if c.endswith("_update")])

    parentless = df.filter(pl.col("parents").is_null())
    if parentless.height > 0:
        for row in parentless.iter_rows(named=True):
            logging.critical(f"No parents exist for {row['id']}")
        logging.critical("This is probably the case, because roompatches were renamed upstream")
        raise RuntimeError("Invariant not preserved")

    return df


def _clean_tumonline_rooms():
    """
    Apply some known corrections / patches on the TUMonline room data.

    It also searches for inconsistencies not yet patched
    """
    rooms = tumonline.Room.load_all()

    with (SOURCES_PATH / "15_patches-rooms_tumonline.yaml").open(encoding="utf-8") as file:
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
        roomcode_split = room_code.split(".")
        if len(roomcode_split) != 3:
            logging.warning(f"Invalid roomcode: Not three '.'-separated parts: {room_code}")
            invalid_rooms.append(room_code)
            continue
        roomcode_parts: tuple[str, str, str] = (roomcode_split[0], roomcode_split[1], roomcode_split[2])
        if len(set(room_code) - ALLOWED_ROOMCODE_CHARS) > 0:
            logging.warning(
                f"Invalid character(s) in roomcode '{room_code}': {set(room_code) - ALLOWED_ROOMCODE_CHARS}",
            )
            invalid_rooms.append(room_code)

        if roomcode_parts[1] not in used_roomcode_levels:
            used_roomcode_levels[roomcode_parts[1]] = room_code

        # Validate the arch_name.
        arch_name_split = room.arch_name.split("@")
        if len(arch_name_split) != 2:
            logging.warning(f"Invalid arch_name: No '@' in '{room.arch_name}' (room {room_code})")
            invalid_rooms.append(room_code)
            continue
        arch_name_parts: tuple[str, str] = (arch_name_split[0], arch_name_split[1])
        if len(arch_name_parts[1]) != 4 or not arch_name_parts[1].isdigit():
            logging.warning(
                f"Invalid building specification in arch_name: Not four digits: "
                f"'{arch_name_parts[1]}' in '{room.arch_name}' (room {room_code})",
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
    if room.arch_name is None or room.arch_name not in used_arch_names:
        if room.arch_name is not None:
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
            looked_up_r2.arch_name = arch_name_parts[0] + r2_parts[2][min_len:].lower() + "@" + arch_name_parts[1]
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
    if room.alt_name is None:
        return
    alt_parts = room.alt_name.split(",")
    if len(alt_parts) < 2:
        return
    if alt_parts[0].lower() == arch_name_parts[0].lower():
        room.alt_name = ", ".join(alt_parts[1:]).strip()
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
    elif "," in arch_name_parts[0] and room.alt_name is not None and room.alt_name.startswith(arch_name_parts[0]):
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
    if room.alt_name is not None:
        room.alt_name = room.alt_name.strip()
