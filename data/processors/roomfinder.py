import orjson
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
                "roomfinder_data_json_new": orjson.dumps(
                    {
                        "b_id": building.b_id,
                        "b_name": building.b_name,
                        "b_alias": building.b_alias,
                        "b_area": building.b_area,
                        "b_room_count": building.b_room_count,
                    }
                ).decode(),
                "sources_rf_json": orjson.dumps(
                    {
                        "name": "Roomfinder",
                        "url": f"https://portal.mytum.de/displayRoomMap?@{building.b_id}",
                    }
                ).decode(),
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


def _rf_source_json(r_id: str) -> str:
    return orjson.dumps(
        {
            "name": "Roomfinder",
            "url": f"https://portal.mytum.de/displayRoomMap?roomid={r_id}&disable_decoration=yes",
        }
    ).decode()


def _rf_data_json(room: "roomfinder.Room") -> str:
    return orjson.dumps(
        {
            "r_alias": room.r_alias,
            "r_number": room.r_number,
            "r_id": room.r_id,
            "r_level": room.r_level,
        }
    ).decode()


def merge_roomfinder_rooms(df: pl.DataFrame) -> pl.DataFrame:
    """
    Merge the rooms in Roomfinder with the existing data.

    Returns a new DataFrame with roomfinder room data merged in.
    """
    with (SOURCES_PATH / "16_roomfinder-merge-patches.yaml").open(encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    # arch_name -> id, derived directly from the JSON-string column.
    arch_name_lookup: dict[str, str] = dict(
        df.filter((pl.col("type") == "room") & pl.col("tumonline_data_json").is_not_null())
        .select(
            pl.col("tumonline_data_json").str.json_path_match("$.arch_name").str.to_lowercase().alias("arch_name"),
            pl.col("id"),
        )
        .filter(pl.col("arch_name").is_not_null())
        .iter_rows(),
    )

    # Minimal id lookup for parent resolution and current-name reads.
    id_lookup: dict[str, dict[str, Any]] = {
        row["id"]: row for row in df.select("id", "parents", "name").iter_rows(named=True)
    }

    updates: dict[str, dict[str, Any]] = {}
    new_rows: list[dict[str, Any]] = []

    for room in roomfinder.Room.load_all():
        try:
            r_id = _find_room_id(room, id_lookup, arch_name_lookup, patches)
            if r_id is None:
                continue
        except RoomNotFoundException as exc:
            if not exc.known_issue:
                logging.warning(exc.message)
                continue
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
                    "data_quality_json": orjson.dumps({"not_in_tumonline": True}).decode(),
                    "roomfinder_data_json": _rf_data_json(room),
                    "coords_lat": room.lat,
                    "coords_lon": room.lon,
                    "coords_source": "roomfinder",
                    "sources_base_json": orjson.dumps(
                        [
                            {
                                "name": "Roomfinder",
                                "url": f"https://portal.mytum.de/displayRoomMap?roomid={room.r_id}&disable_decoration=yes",
                            }
                        ]
                    ).decode(),
                }
            )
            id_lookup[r_id] = {"id": r_id, "parents": parents, "name": name}
            continue

        upd = updates.setdefault(r_id, {})

        current_name = id_lookup.get(r_id, {}).get("name", "")
        if current_name and "(" not in current_name and len(room.r_alias) > 0:
            new_name = f"{current_name} ({room.r_alias})"
            upd["name"] = new_name
            upd["name_de"] = new_name
            upd["name_en"] = new_name

        # First-roomfinder-wins for data + coords
        if "roomfinder_data_json" not in upd:
            upd["roomfinder_data_json"] = _rf_data_json(room)
            upd["coords_lat_rf"] = room.lat
            upd["coords_lon_rf"] = room.lon

        # Sources are appended for every matching roomfinder room
        src = _rf_source_json(room.r_id)
        upd["sources_rf_json"] = src if "sources_rf_json" not in upd else f"{upd['sources_rf_json']},{src}"

    if updates:
        upd_df = pl.DataFrame(
            [{"id": uid, **ufields} for uid, ufields in updates.items()],
            infer_schema_length=None,
        )
        df = df.join(upd_df, on="id", how="left", suffix="_upd")

        # Names: update wins when present
        for col in ("name", "name_de", "name_en"):
            upd_col = f"{col}_upd"
            if upd_col in df.columns:
                df = df.with_columns(pl.coalesce(pl.col(upd_col), pl.col(col)).alias(col)).drop(upd_col)

        # roomfinder_data_json: overwrite when present
        if "roomfinder_data_json_upd" in df.columns:
            df = df.with_columns(
                pl.coalesce(pl.col("roomfinder_data_json_upd"), pl.col("roomfinder_data_json")).alias(
                    "roomfinder_data_json"
                ),
            ).drop("roomfinder_data_json_upd")

        # Coords: setdefault, and tag coords_source when we actually used roomfinder
        if "coords_lat_rf" in df.columns:
            df = df.with_columns(
                pl.coalesce(pl.col("coords_lat"), pl.col("coords_lat_rf")).alias("coords_lat"),
                pl.coalesce(pl.col("coords_lon"), pl.col("coords_lon_rf")).alias("coords_lon"),
                pl.when(pl.col("coords_source").is_null() & pl.col("coords_lat_rf").is_not_null())
                .then(pl.lit("roomfinder"))
                .otherwise(pl.col("coords_source"))
                .alias("coords_source"),
            ).drop("coords_lat_rf", "coords_lon_rf")

        # Append Roomfinder sources to sources_base_json
        if "sources_rf_json" in df.columns:
            df = df.with_columns(
                pl.when(pl.col("sources_rf_json").is_not_null())
                .then(
                    pl.when(pl.col("sources_base_json").is_not_null())
                    .then(pl.col("sources_base_json").str.strip_suffix("]") + "," + pl.col("sources_rf_json") + "]")
                    .otherwise("[" + pl.col("sources_rf_json") + "]"),
                )
                .otherwise(pl.col("sources_base_json"))
                .alias("sources_base_json"),
            ).drop("sources_rf_json")

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
