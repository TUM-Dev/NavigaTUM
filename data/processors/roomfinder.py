import json
import logging
import re
from pathlib import Path
from typing import Any

import polars as pl
import yaml
from external.models import roomfinder
from processors.df_utils import ensure_column

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"


def merge_roomfinder_buildings(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge the buildings in Roomfinder with the existing data.

    Returns a new DataFrame with roomfinder building data merged in.
    """
    with (SOURCES_PATH / "10_patches-roomfinder-buildings.yaml").open(encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    error = False
    rf_rows = []
    seen_b_ids: dict[str, int] = {}

    for building in roomfinder.Building.load_all():
        # 'Building' 0000 contains some buildings and places not in TUMonline as rooms.
        # They might be integrated customly somewhere else, but here we ignore these.
        if building.b_id == "0000":
            continue

        for wrong, correct in patches["replacements"].items():
            if building.b_id == wrong:
                building.b_id = correct

        # Check for duplicate b_prefix matches in the input DataFrame
        matches = df.filter(pl.col("b_prefix") == building.b_id)
        if len(matches) == 0:
            # The Roomfinder appears to be no longer maintained, so sometimes there are still
            # buildings in it that no longer exist. Previously this was an error, but for this
            # reason now it is a warning.
            logging.warning(f"building '{building.b_id}' not found in base data. It may be missing in the areatree.")
            continue
        if len(matches) > 1:
            logging.error(f"building id '{building.b_id}' more than once in base data")
            error = True
            continue

        if building.b_id in seen_b_ids:
            # Already processed this b_id (due to replacement mapping)
            continue
        seen_b_ids[building.b_id] = 1

        rf_rows.append(
            {
                "b_prefix_match": building.b_id,
                "roomfinder_data_json_new": json.dumps(
                    {
                        "b_id": building.b_id,
                        "b_name": building.b_name,
                        "b_alias": building.b_alias,
                        "b_area": building.b_area,
                        "b_room_count": building.b_room_count,
                    }
                ),
                "sources_rf_json": json.dumps(
                    {
                        "name": "Roomfinder",
                        "url": f"https://portal.mytum.de/displayRoomMap?@{building.b_id}",
                    }
                ),
                "coords_lat_rf": building.lat,
                "coords_lon_rf": building.lon,
                "props_ids_b_id_rf": building.b_id,
            }
        )

    if error:
        raise RuntimeError("One or more errors, aborting")

    if not rf_rows:
        return df

    rf_df = pl.DataFrame(rf_rows)

    # Ensure columns exist before coalescing
    for col in ["roomfinder_data_json", "coords_lat", "coords_lon", "coords_source", "props_ids_b_id"]:
        df = ensure_column(df, col)

    # Join on b_prefix
    result = df.join(rf_df, left_on="b_prefix", right_on="b_prefix_match", how="left")

    # Coalesce: don't overwrite existing values (setdefault semantics)
    result = result.with_columns(
        [
            pl.coalesce(pl.col("roomfinder_data_json"), pl.col("roomfinder_data_json_new")).alias(
                "roomfinder_data_json"
            ),
            pl.coalesce(pl.col("coords_lat"), pl.col("coords_lat_rf")).alias("coords_lat"),
            pl.coalesce(pl.col("coords_lon"), pl.col("coords_lon_rf")).alias("coords_lon"),
            # Only set roomfinder source when we actually used roomfinder coords (existing was null)
            pl.when(pl.col("coords_lat").is_null() & pl.col("coords_lat_rf").is_not_null())
            .then(pl.lit("roomfinder"))
            .otherwise(pl.col("coords_source"))
            .alias("coords_source"),
            pl.coalesce(pl.col("props_ids_b_id"), pl.col("props_ids_b_id_rf")).alias("props_ids_b_id"),
        ]
    )

    # Append Roomfinder to sources_base_json where matched
    result = result.with_columns(
        pl.when(pl.col("sources_rf_json").is_not_null())
        .then(
            pl.when(pl.col("sources_base_json").is_not_null())
            .then(pl.col("sources_base_json").str.strip_suffix("]") + "," + pl.col("sources_rf_json") + "]")
            .otherwise("[" + pl.col("sources_rf_json") + "]"),
        )
        .otherwise(pl.col("sources_base_json"))
        .alias("sources_base_json"),
    )

    result = result.drop(
        ["roomfinder_data_json_new", "sources_rf_json", "coords_lat_rf", "coords_lon_rf", "props_ids_b_id_rf"]
    )
    return result


def merge_roomfinder_rooms(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge the rooms in Roomfinder with the existing data.

    Returns a new DataFrame with roomfinder room data merged in.
    """
    with (SOURCES_PATH / "16_roomfinder-merge-patches.yaml").open(encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    # Build arch_name lookup from df: arch_name -> id
    room_rows = (
        df.filter(
            (pl.col("type") == "room") & pl.col("tumonline_data_json").is_not_null(),
        )
        .select("id", "tumonline_data_json")
        .to_dicts()
    )

    arch_name_lookup: dict[str, str] = {}
    for row in room_rows:
        td = json.loads(row["tumonline_data_json"])
        if td.get("arch_name"):
            arch_name_lookup[td["arch_name"].lower()] = row["id"]

    # Build id-based lookups from df for parent resolution and name updates
    id_lookup: dict[str, dict[str, Any]] = {}
    for row in df.select("id", "parents", "name").to_dicts():
        id_lookup[row["id"]] = row

    # Collect updates for existing rows and new rows
    updates: dict[str, dict[str, Any]] = {}  # id -> fields to update
    new_rows: list[dict[str, Any]] = []

    for room in roomfinder.Room.load_all():
        # Try to find the existing room id (which is based on the SAP Code).
        try:
            r_id = _find_room_id(room, id_lookup, arch_name_lookup, patches)
            if r_id is None:
                continue
        except RoomNotFoundException as exc:
            if exc.known_issue:
                r_id = patches["known_issues"]["not_in_tumonline"][room.r_id]
                parent_row = id_lookup.get(room.b_id)
                if parent_row is None:
                    logging.warning(f"Parent building '{room.b_id}' not found for room '{room.r_id}'")
                    continue
                parents = parent_row["parents"] + [room.b_id]
                name = r_id if len(room.r_alias) == 0 else f"{r_id} ({room.r_alias})"
                new_rows.append(
                    {
                        "id": r_id,
                        "type": "room",
                        "name": name,
                        "name_de": name,
                        "name_en": name,
                        "parents": parents,
                        "data_quality_json": json.dumps({"not_in_tumonline": True}),
                        "roomfinder_data_json": json.dumps(
                            {
                                "r_alias": room.r_alias,
                                "r_number": room.r_number,
                                "r_id": room.r_id,
                                "r_level": room.r_level,
                            }
                        ),
                        "coords_lat": room.lat,
                        "coords_lon": room.lon,
                        "coords_source": "roomfinder",
                        "sources_base_json": json.dumps(
                            [
                                {
                                    "name": "Roomfinder",
                                    "url": f"https://portal.mytum.de/displayRoomMap?roomid={room.r_id}&disable_decoration=yes",
                                }
                            ]
                        ),
                    }
                )
                # Register new row in id_lookup so subsequent rooms can reference it
                id_lookup[r_id] = {"id": r_id, "parents": parents, "name": name}
            else:
                logging.warning(exc.message)
                continue

        if r_id not in updates:
            updates[r_id] = {}

        upd = updates[r_id]

        # Update name with alias if not already present
        current_name = id_lookup.get(r_id, {}).get("name", "")
        if current_name and "(" not in current_name and len(room.r_alias) > 0:
            new_name = f"{current_name} ({room.r_alias})"
            upd["name"] = new_name
            upd["name_de"] = new_name
            upd["name_en"] = new_name

        # First roomfinder room wins for data and coords (matches original setdefault behavior)
        if "roomfinder_data_json" not in upd:
            upd["roomfinder_data_json"] = json.dumps(
                {
                    "r_alias": room.r_alias,
                    "r_number": room.r_number,
                    "r_id": room.r_id,
                    "r_level": room.r_level,
                }
            )
            upd["coords_lat_rf"] = room.lat
            upd["coords_lon_rf"] = room.lon

        # Always append source (original code appended for every matching roomfinder room)
        source_json = json.dumps(
            {
                "name": "Roomfinder",
                "url": f"https://portal.mytum.de/displayRoomMap?roomid={room.r_id}&disable_decoration=yes",
            }
        )
        if "sources_rf_json" not in upd:
            upd["sources_rf_json"] = source_json
        else:
            upd["sources_rf_json"] = upd["sources_rf_json"] + "," + source_json

    # Apply updates to existing rows
    if updates:
        update_rows = [{"id": uid, **ufields} for uid, ufields in updates.items()]
        upd_df = pl.DataFrame(update_rows, infer_schema_length=None)

        # Join updates
        result = df.join(upd_df, on="id", how="left", suffix="_upd")

        # Apply name updates
        if "name_upd" in result.columns:
            result = result.with_columns(
                pl.coalesce(pl.col("name_upd"), pl.col("name")).alias("name"),
            )
            result = result.drop("name_upd")
        if "name_de_upd" in result.columns:
            result = result.with_columns(
                pl.coalesce(pl.col("name_de_upd"), pl.col("name_de")).alias("name_de"),
            )
            result = result.drop("name_de_upd")
        if "name_en_upd" in result.columns:
            result = result.with_columns(
                pl.coalesce(pl.col("name_en_upd"), pl.col("name_en")).alias("name_en"),
            )
            result = result.drop("name_en_upd")

        # Roomfinder data: overwrite (not setdefault — original code overwrites)
        if "roomfinder_data_json_upd" in result.columns:
            result = result.with_columns(
                pl.coalesce(pl.col("roomfinder_data_json_upd"), pl.col("roomfinder_data_json")).alias(
                    "roomfinder_data_json"
                ),
            )
            result = result.drop("roomfinder_data_json_upd")

        # Coords: setdefault (don't overwrite existing)
        if "coords_lat_rf" in result.columns:
            result = result.with_columns(
                [
                    pl.coalesce(pl.col("coords_lat"), pl.col("coords_lat_rf")).alias("coords_lat"),
                    pl.coalesce(pl.col("coords_lon"), pl.col("coords_lon_rf")).alias("coords_lon"),
                    pl.when(pl.col("coords_source").is_null() & pl.col("coords_lat_rf").is_not_null())
                    .then(pl.lit("roomfinder"))
                    .otherwise(pl.col("coords_source"))
                    .alias("coords_source"),
                ]
            )
            result = result.drop(["coords_lat_rf", "coords_lon_rf"])

        # Sources: append Roomfinder
        if "sources_rf_json" in result.columns:
            result = result.with_columns(
                pl.when(pl.col("sources_rf_json").is_not_null())
                .then(
                    pl.when(pl.col("sources_base_json").is_not_null())
                    .then(pl.col("sources_base_json").str.strip_suffix("]") + "," + pl.col("sources_rf_json") + "]")
                    .otherwise("[" + pl.col("sources_rf_json") + "]"),
                )
                .otherwise(pl.col("sources_base_json"))
                .alias("sources_base_json"),
            )
            result = result.drop("sources_rf_json")

        df = result

    # Add new rows
    if new_rows:
        new_df = pl.DataFrame(new_rows, infer_schema_length=None)
        df = pl.concat([df, new_df], how="diagonal_relaxed")

    return df


def _find_room_id(
    room: roomfinder.Room, id_lookup: dict[str, Any], arch_name_lookup: dict[str, str], patches: dict[str, Any]
) -> str | None:
    if room.r_id in patches["ignore"]:
        return None

    if room.r_id in patches["known_issues"]["mapping"]:
        return str(patches["known_issues"]["mapping"][room.r_id])

    if room.r_id in patches["known_issues"]["not_in_tumonline"]:
        raise RoomNotFoundException(known_issue=True)

    # Verify first, that the building is included in the data.
    # Buildings not in the data are ignored.
    if room.b_id not in id_lookup:
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
