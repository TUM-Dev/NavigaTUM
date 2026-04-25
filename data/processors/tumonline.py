import json
import logging
import string
from pathlib import Path
from typing import Any

import polars as pl
import yaml

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
                "tumonline_data_json_new": json.dumps(
                    {"name": b_name, "filter_id": building.filter_id, "area_id": building.area_id},
                    ensure_ascii=False,
                ),
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


# pylint: disable=too-many-locals
def merge_tumonline_rooms(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge the rooms in TUMonline with the existing data.

    Returns a new DataFrame with tumonline room data merged in.
    """
    rooms = _clean_tumonline_rooms()

    orgs_de = tumonline.Organisation.load_all_for("de")
    orgs_en = tumonline.Organisation.load_all_for("en")
    usages_lookup = tumonline.Usage.load_all()
    org_id_to_code = {key: org.code for key, org in orgs_de.items()}

    # Build lookup of existing IDs and building parents from df
    existing_ids = set(df["id"].to_list())
    # Build a dict of building_id -> parents list for quick lookup
    building_rows = df.filter(pl.col("type").is_in(["building", "joined_building"])).select("id", "parents")
    building_parents_lookup: dict[str, list[str]] = {}
    for brow in building_rows.iter_rows(named=True):
        building_parents_lookup[brow["id"]] = brow["parents"] if brow["parents"] is not None else []

    new_room_rows: list[dict[str, Any]] = []
    update_room_rows: list[dict[str, Any]] = []
    missing_buildings: dict[str, int] = {}

    for room_code, room in rooms.items():
        # Extract building id
        b_id = room_code.split(".")[0]
        if b_id not in building_parents_lookup:
            missing_buildings.setdefault(b_id, 0)
            missing_buildings[b_id] += 1
            continue

        parents = building_parents_lookup[b_id] + [b_id]

        # Build operator_name
        operator_name = None
        if room.main_operator_id in orgs_de:
            operator_name = _(
                orgs_de[room.main_operator_id].name,
                orgs_en[room.main_operator_id].name,
            )

        tumonline_data = {
            "tumonline_id": room.tumonline_id,
            "roomcode": room_code,
            "arch_name": room.arch_name,
            "alt_name": room.alt_name,
            "address": {"place": room.address.place, "street": room.address.street, "zip_code": room.address.zip_code},
            "operator": org_id_to_code.get(room.main_operator_id),
            "operator_id": room.main_operator_id,
            "operator_name": operator_name,
            "calendar": room.calendar_resource_nr,
            "usage": room.usage_id,
        }

        # Usage
        if room.usage_id in usages_lookup:
            tumonline_usage = usages_lookup[room.usage_id]
            usage_name = _(tumonline_usage.name)
        else:
            logging.error(f"Unknown usage for room '{room_code}': Id '{room.usage_id}'")
            continue

        # Build source info
        source_entry = {
            "name": "TUMonline",
            "url": f"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/wbRaum.editRaum?pRaumNr={room.tumonline_id}",
        }

        row: dict[str, Any] = {
            "id": room_code,
            "type": "room",
            "name": f"{room_code} ({room.alt_name})",
            "name_de": f"{room_code} ({room.alt_name})",
            "name_en": f"{room_code} ({room.alt_name})",
            "parents": parents,
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
            "usage_din_277": tumonline_usage.din277_id,
            "usage_din_277_desc": tumonline_usage.din277_name,
            "sources_base_json": to_json_or_none([source_entry]),
            "sources_patched": True if room.patched else None,
        }

        if room_code in existing_ids:
            update_room_rows.append(row)
        else:
            new_room_rows.append(row)

    # Handle truly new rooms: concat
    if new_room_rows:
        new_rooms_df = pl.DataFrame(new_room_rows, infer_schema_length=None)
        # Ensure bool columns are correctly typed (may be inferred as Null if first rows are None)
        if "sources_patched" in new_rooms_df.columns and new_rooms_df.schema["sources_patched"] == pl.Null:
            new_rooms_df = new_rooms_df.with_columns(pl.col("sources_patched").cast(pl.Boolean))
        df = pl.concat([df, new_rooms_df], how="diagonal_relaxed")

    # Handle updates: existing rows should NOT be overwritten (overwrite=False semantics)
    # For each update row, only fill in columns that are currently null in the existing row
    if update_room_rows:
        updates_df = pl.DataFrame(update_room_rows)
        # Suffix all columns except 'id' for the join
        update_cols = [c for c in updates_df.columns if c != "id"]

        # Ensure all update columns exist in df before joining
        df = ensure_columns(df, {col: updates_df.schema.get(col, pl.Utf8()) for col in update_cols})

        df = df.join(
            updates_df,
            on="id",
            how="left",
            suffix="_update",
        )

        # For each column, coalesce: existing value wins (overwrite=False)
        coalesce_exprs = []
        for col in update_cols:
            update_col = f"{col}_update"
            if update_col in df.columns:
                coalesce_exprs.append(pl.coalesce(pl.col(col), pl.col(update_col)).alias(col))
        if coalesce_exprs:
            df = df.with_columns(coalesce_exprs)

        # Batch-append TUMonline source to existing sources_base_json for updated rows
        source_updates = []
        for urow in update_room_rows:
            source_updates.append(
                {
                    "id": urow["id"],
                    "sources_new_json": urow["sources_base_json"],
                    "patched_flag": urow.get("sources_patched") or False,
                }
            )
        if source_updates:
            src_df = pl.DataFrame(
                source_updates, schema={"id": pl.Utf8, "sources_new_json": pl.Utf8, "patched_flag": pl.Boolean}
            )
            df = df.join(src_df, on="id", how="left")
            df = ensure_column(df, "sources_patched", pl.Boolean())
            df = df.with_columns(
                # Append new source to existing sources_base_json
                pl.when(pl.col("sources_new_json").is_not_null() & pl.col("sources_base_json").is_not_null())
                .then(
                    pl.col("sources_base_json").str.strip_suffix("]")
                    + ","
                    + pl.col("sources_new_json").str.strip_prefix("[")
                )
                .when(pl.col("sources_new_json").is_not_null())
                .then(pl.col("sources_new_json"))
                .otherwise(pl.col("sources_base_json"))
                .alias("sources_base_json"),
                # Set patched flag
                pl.when(pl.col("patched_flag") == True)  # noqa: E712
                .then(pl.lit(True))
                .otherwise(pl.col("sources_patched"))
                .alias("sources_patched"),
            )
            df = df.drop(["sources_new_json", "patched_flag"])

        # Drop the _update columns
        drop_cols = [c for c in df.columns if c.endswith("_update")]
        if drop_cols:
            df = df.drop(drop_cols)

    # Validate: all entries must have parents
    parentless = df.filter(pl.col("parents").is_null())
    if parentless.height > 0:
        for row in parentless.iter_rows(named=True):
            logging.critical(f"No parents exist for {row['id']}")
        logging.critical("This is probably the case, because roompatches were renamed upstream")
        raise RuntimeError("Invariant not preserved")

    if missing_buildings:
        logging.warning(
            f"Ignored {sum(missing_buildings.values())} rooms for the following buildings, "
            f"which were not found: {sorted(missing_buildings.keys())}",
        )

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
