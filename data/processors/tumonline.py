import logging
import re
import string
from pathlib import Path
from typing import Any

import orjson
import polars as pl
import yaml
from external.loaders.tumonline import load_buildings, load_orgs, load_rooms, load_usages
from utils import TranslatableStr as _

from processors.df_utils import ensure_column, ensure_columns, to_json_or_none, translatable_to_columns
from processors.patch import apply_roomcode_patch

_logger = logging.getLogger(__name__)

ALLOWED_ROOMCODE_CHARS = set(string.ascii_letters) | set(string.digits) | {".", "-"}
OPERATOR_STRIP_CHARS = "[ ]"
OPERATOR_WEBNAV_LINK_PREFIX = "webnav.navigate_to?corg="

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"

# TUMonline truncates the building name column to 40 characters, so any name at exactly that
# length is treated as truncated when comparing.
_TUMONLINE_NAME_DB_LIMIT = 40
# Leading parenthetical building codes that TUMonline prepends (e.g. "(N1) Hörsäle (U-Trakt)" or
# "(Südost 1) ..."). NavigaTUM's areatree drops these in favour of an explicit visible_id.
_TUMONLINE_LEADING_CODE_RE = re.compile(r"^\((?:[A-Z]{1,3}\d{0,2}[A-Za-z]?|Südost\s*\d+)\)\s+")
# TUMonline operator/location markers — allowed anywhere in the name, not just trailing
# (e.g. "Neherstr.1 (AM) ForTe", "Prinzregentenstr. 68 (AM) MRI", "Heßstr. 134 (UMBAU)").
_TUMONLINE_TRAILING_NOISE_RE = re.compile(r"\s*\((?:AM|NR|SZ|GP|GM|LfL|HSWT|UMBAU|VSG\.\w+|VST\.\w+)\)\s*")
# Trailing operator-suffix tokens TUMonline appends to building names.
_TUMONLINE_TRAILING_OPERATOR_RE = re.compile(r"\s+(?:LMU|PH|MRI|ForTe)\s*$", re.IGNORECASE)
# Location/operator prefixes TUMonline prepends that the areatree drops because the parent
# area already supplies that context (e.g. an "Obernach" sub-area contains a "Bürogebäude"
# entry; TUMonline reports it as "Obernach Bürogebäude").
_TUMONLINE_LOCATION_PREFIX_RE = re.compile(
    r"^(?:"
    r"Obernach|Veitshof|Dürnast|Viehhausen|Starnberg"
    r"|VST\.Thalh\.-|VSG\.(?:Grünschw|Roggst|Roggenst)\.-"
    r"|Holzforschung München|Limnologische Station"
    r"|FMI/|LfL|GP|TUM CS|TUMCS|HSWT|ZIP\s*-"
    r"|GM(?=[A-Z])"  # "GMTrafostation" — strip just "GM" when no space follows
    r"|GM"  # "GM Verwaltung" — strip with optional trailing space
    r"|Garmisch-Partenkirchen"
    r")[ ]*"
)
# Common TUMonline abbreviations expanded so they compare equal to the spelled-out areatree form.
_ABBREV_EXPANSIONS: list[tuple[re.Pattern[str], str]] = [
    # ``str.`` appears inside compounds (``Bahnhofstr.``) so it needs no word boundary.
    (re.compile(r"str\.", re.IGNORECASE), "straße"),
    (re.compile(r"\bstrasse\b", re.IGNORECASE), "straße"),
    (re.compile(r"\bf\.\s?", re.IGNORECASE), "für "),
    (re.compile(r"\bu\.\s?", re.IGNORECASE), "und "),
    (re.compile(r"\bintern\.\s?", re.IGNORECASE), "internationales "),
    (re.compile(r"\binst\.\s?", re.IGNORECASE), "institut "),
    (re.compile(r"\bgeb\.\s?", re.IGNORECASE), "gebäude "),
    (re.compile(r"\bwerkst\.\s?", re.IGNORECASE), "werkstatt "),
    (re.compile(r"\behm\.\s?", re.IGNORECASE), "ehemaliger "),
]
# Trailing short-name parenthetical NavigaTUM uses but TUMonline does not.
_AREATREE_TRAILING_SHORT_RE = re.compile(
    r"\s*\((?:"
    r"Bau\s+\d+|BL\.?\s*[A-Z]|BT\d+|CH\s*\d+|MW\s*\d+|SG\s*\d+|PG\s*\d+"
    r"|[A-Z]{1,3}\s?\d+[A-Z]?|[A-Z]{2,6}(?:[-/][A-Z]{2,6})?"
    r")\)\s*$"
)
# Areatree-side leading "Gebäudeteil N, " enrichment — drop so the comparison hinges on the
# real building name rather than the descriptor.
_AREATREE_LEADING_GEBAEUDETEIL_RE = re.compile(r"^Gebäudeteil\s+\d+,\s*", re.IGNORECASE)


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
            _logger.warning(f"building id '{b_id}' ('{b_name}') more than once in base data")
            error = True
            continue
        if matches.height == 0:
            # Currently, not an error, because the areatree is built by hand.
            _logger.warning(f"building id '{b_id}' ('{b_name}') not found in base data, ignoring")
            continue

        match_row = matches.row(0, named=True)
        areatree_name = match_row.get("name")
        areatree_short_name = match_row.get("short_name")
        if areatree_name and not _building_names_equivalent(areatree_name, b_name, areatree_short_name):
            _logger.warning(
                f"building id '{b_id}': name in areatree ('{areatree_name}') differs from TUMonline ('{b_name}')"
            )

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
    return result.drop(["tumonline_data_json_new", "props_ids_b_id_new"])


def _alphanum_lower(s: str) -> str:
    """Reduce a name to comparable form: expand abbrevs, then keep lowercase alphanumerics only."""
    s = s.lower()
    for pattern, replacement in _ABBREV_EXPANSIONS:
        s = pattern.sub(replacement, s)
    return re.sub(r"[^a-z0-9äöüß]+", "", s)


def _strip_noise(name: str, *, drop_location_prefix: bool) -> str:
    """Apply all known noise-stripping patterns to a name (works for either side)."""
    n = _TUMONLINE_LEADING_CODE_RE.sub("", name)
    if drop_location_prefix:
        n = _TUMONLINE_LOCATION_PREFIX_RE.sub("", n)
    n = _TUMONLINE_TRAILING_NOISE_RE.sub(" ", n)
    n = _TUMONLINE_TRAILING_OPERATOR_RE.sub("", n)
    n = _AREATREE_TRAILING_SHORT_RE.sub("", n)
    n = _AREATREE_LEADING_GEBAEUDETEIL_RE.sub("", n)
    return n.strip()


def _building_names_equivalent(
    areatree_name: str,
    tumonline_name: str,
    areatree_short_name: str | None = None,
) -> bool:
    """
    Decide whether the two names refer to the same building modulo known noise.

    Suppresses warnings caused by:
    - TUMonline's leading building-code prefix (e.g. ``(N1) U-Trakt``).
    - TUMonline's location/operator prefix (``Obernach``, ``Dürnast``, ``FMI/``, ``LfL`` ...).
      The areatree's parent area already provides this context — but if the areatree itself
      includes the prefix in the name, we must compare without stripping. So we try both ways.
    - TUMonline's operator/location markers (``(AM)``, ``(NR)``, ``(SZ)``, ``(UMBAU)`` ...) —
      anywhere in the string, not just trailing.
    - TUMonline's trailing operator-suffix tokens (``LMU``, ``PH``, ``MRI``, ``ForTe``).
    - Common German abbreviations (``Str.`` ↔ ``Straße``, ``f.`` ↔ ``für``, ``u.`` ↔ ``und``,
      ``Inst.`` ↔ ``Institut``, ``Geb.`` ↔ ``Gebäude``, ``Werkst.`` ↔ ``Werkstatt``).
    - NavigaTUM's trailing short-name / building-code parentheticals (``(WSI)``, ``(Bau 501)``).
    - TUMonline's 40-character database truncation.
    - The areatree's explicit ``|short_name``: if TUMonline's name boils down to the short_name
      (e.g. AT ``Petersgasse 18|PG 18`` vs TUMonline ``TUM CS PG18``), treat as equivalent.
    """
    short_norm = _alphanum_lower(areatree_short_name) if areatree_short_name else ""
    # Try both with and without the TUMonline location prefix — the areatree may or may not carry it.
    for drop_prefix in (True, False):
        a_norm = _alphanum_lower(_strip_noise(areatree_name, drop_location_prefix=drop_prefix))
        t_norm = _alphanum_lower(_strip_noise(tumonline_name, drop_location_prefix=drop_prefix))
        if a_norm == t_norm:
            return True
        # Areatree commonly enriches the TUMonline name with an extra suffix (address, descriptor):
        # if AT's normalised form starts with TUM's, accept as equivalent.
        if t_norm and a_norm.startswith(t_norm) and len(t_norm) >= 5:
            return True
        # If TUMonline's name reduces to the areatree's explicit short_name, accept.
        if short_norm and t_norm == short_norm:
            return True
        # 40-char truncation: accept either direction — the areatree name might be the full
        # canonical form (TUM is its truncated prefix) or the areatree name might match the
        # truncated prefix itself (allowing 1-2 partial trailing chars).
        if len(tumonline_name) == _TUMONLINE_NAME_DB_LIMIT:
            if a_norm and t_norm.startswith(a_norm):
                return True
            tail = max(1, len(t_norm) - 2)
            if t_norm and a_norm.startswith(t_norm[:tail]):
                return True
    return False


_BUILDING_TYPES = ["building", "joined_building"]



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

        if room["usage_id"] not in usages_lookup:
            _logger.error(f"Unknown usage for room '{room_code}': Id '{room['usage_id']}'")
            continue
        tumonline_usage = usages_lookup[room["usage_id"]]
        usage_name = _(tumonline_usage["name"])

        op = orgs_lookup.get(room["main_operator_id"])
        operator_name = _(op["name_de"], op["name_en"]) if op else None

        tumonline_data = {
            "tumonline_id": room["tumonline_id"],
            "roomcode": room_code,
            "arch_name": room["arch_name"],
            "alt_name": room["alt_name"],
            "address": {
                "place": room["address_place"],
                "street": room["address_street"],
                "zip_code": room["address_zip_code"],
            },
            "operator": op["code"] if op else None,
            "operator_id": room["main_operator_id"],
            "operator_name": operator_name,
            "calendar": room["calendar_resource_nr"],
            "usage": room["usage_id"],
        }
        source_entry = {
            "name": "TUMonline",
            "url": f"https://campus.tum.de/tumonline/ee/ui/ca2/app/desktop/#/pl/ui/$ctx/wbRaum.editRaum?pRaumNr={room['tumonline_id']}",
        }
        candidate_rows.append(
            {
                "id": room_code,
                "type": "room",
                "name": f"{room_code} ({room['alt_name']})",
                "name_de": f"{room_code} ({room['alt_name']})",
                "name_en": f"{room_code} ({room['alt_name']})",
                "parents": building_parents[b_id] + [b_id],
                "tumonline_data_json": to_json_or_none(tumonline_data),
                "props_ids_roomcode": room_code,
                "props_ids_arch_name": room["arch_name"] if room["arch_name"] else None,
                "props_address_street": room["address_street"],
                "props_address_plz_place": f"{room['address_zip_code']} {room['address_place']}",
                "props_address_source": "tumonline",
                "props_stats_n_seats_sitting": room["seats_sitting"],
                "props_stats_n_seats_standing": room["seats_standing"],
                "props_stats_n_seats_wheelchair": room["seats_wheelchair"],
                **translatable_to_columns("usage_name", usage_name),
                "usage_din_277": tumonline_usage["din277_id"],
                "usage_din_277_desc": tumonline_usage["din277_name"],
                "sources_base_json": to_json_or_none([source_entry]),
                "sources_patched": True if room["patched"] else None,
            }
        )

    if missing_buildings:
        _logger.warning(
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
                .then(pl.lit(True))  # noqa: FBT003
                .otherwise(pl.col("sources_patched"))
                .alias("sources_patched"),
            )
            df = df.drop([c for c in df.columns if c.endswith("_update")])

    parentless = df.filter(pl.col("parents").is_null())
    if parentless.height > 0:
        for row in parentless.iter_rows(named=True):
            _logger.critical(f"No parents exist for {row['id']}")
        _logger.critical("This is probably the case, because roompatches were renamed upstream")
        raise RuntimeError("Invariant not preserved")

    return df


def _clean_tumonline_rooms() -> dict[str, dict[str, Any]]:
    """
    Apply some known corrections / patches on the TUMonline room data.

    It also searches for inconsistencies not yet patched
    """
    rooms: dict[str, dict[str, Any]] = {row["room_key"]: row for row in load_rooms().iter_rows(named=True)}

    with (SOURCES_PATH / "15_patches-rooms_tumonline.yaml").open(encoding="utf-8") as file:
        patches = yaml.safe_load(file.read())

    apply_roomcode_patch(rooms, patches["patches"])

    used_arch_names: dict[str, tuple[str, str, str]] = {}
    used_roomcode_levels = {}
    invalid_rooms: list[str] = []
    for room_code, room in rooms.items():
        if not room["arch_name"] or not room["alt_name"]:
            _logger.warning(
                f"ignoring {room_code} as it has arch_name={room['arch_name']!r}, alt_name={room['alt_name']!r}"
            )
            invalid_rooms.append(room_code)
            continue
        # Validate the room_code
        roomcode_split = room_code.split(".")
        if len(roomcode_split) != 3:
            _logger.warning(f"Invalid roomcode: Not three '.'-separated parts: {room_code}")
            invalid_rooms.append(room_code)
            continue
        roomcode_parts: tuple[str, str, str] = (roomcode_split[0], roomcode_split[1], roomcode_split[2])
        if len(set(room_code) - ALLOWED_ROOMCODE_CHARS) > 0:
            _logger.warning(
                f"Invalid character(s) in roomcode '{room_code}': {set(room_code) - ALLOWED_ROOMCODE_CHARS}",
            )
            invalid_rooms.append(room_code)

        if roomcode_parts[1] not in used_roomcode_levels:
            used_roomcode_levels[roomcode_parts[1]] = room_code

        # Validate the arch_name.
        arch_name_split = room["arch_name"].split("@")
        if len(arch_name_split) != 2:
            _logger.warning(f"Invalid arch_name: No '@' in '{room['arch_name']}' (room {room_code})")
            invalid_rooms.append(room_code)
            continue
        arch_name_parts: tuple[str, str] = (arch_name_split[0], arch_name_split[1])
        if len(arch_name_parts[1]) != 4 or not arch_name_parts[1].isdigit():
            _logger.warning(
                f"Invalid building specification in arch_name: Not four digits: "
                f"'{arch_name_parts[1]}' in '{room['arch_name']}' (room {room_code})",
            )
            invalid_rooms.append(room_code)

        _infer_arch_name(room, arch_name_parts, used_arch_names, roomcode_parts, rooms)

    if invalid_rooms:
        _logger.warning(f"Ignored {len(invalid_rooms)} TUMonline rooms because they are invalid.")
        for room_code in invalid_rooms:
            rooms.pop(room_code)

    return rooms


def _infer_arch_name(
    room: dict[str, Any],
    arch_name_parts: tuple[str, str],
    used_arch_names: dict[str, tuple[str, str, str]],
    roomcode_parts: tuple[str, str, str],
    rooms: dict[str, dict[str, Any]],
) -> None:
    """Infer the arch name and other related properties"""
    # Some rooms don't have an arch_name. The value is then usually just like "@1234".
    # Since this is not helpful (the building is already known for all rooms) rooms are then dropped.
    if len(arch_name_parts[0]) == 0:
        room["arch_name"] = ""
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
            # _logger.debug(f"{r["arch_name"]=}, {roomcode_parts[2]=}")
            pass

    # Check for duplicate uses of arch names
    if room["arch_name"] is None or room["arch_name"] not in used_arch_names:
        if room["arch_name"] is not None:
            used_arch_names[room["arch_name"]] = roomcode_parts
        return

    r1_parts = roomcode_parts
    r2_parts = used_arch_names[room["arch_name"]]
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
            room["arch_name"] = arch_name_parts[0] + r1_parts[2][min_len:].lower() + "@" + arch_name_parts[1]
        else:
            looked_up_r2 = rooms[".".join(used_arch_names[room["arch_name"]])]
            looked_up_r2["arch_name"] = arch_name_parts[0] + r2_parts[2][min_len:].lower() + "@" + arch_name_parts[1]
            used_arch_names[room["arch_name"]] = roomcode_parts
        room["patched"] = True


def _maybe_set_alt_name(room_code: str, arch_name_parts: tuple[str, str], room: dict[str, Any]) -> None:
    """
    Deduces the alt_name from the roomname

    The alt_name commonly begins with the roomname.
    Since ther roomname should be encoded in the arch_name as the part before the "@" we verify,
    that these match and remove the roomname in the alt_name (which is more like a descriptive name).
    As far as we observed so far, if the room has no arch_name it also doesn't have any roomname in the alt_name.
    Also, if there are no comma-separated parts, the roomname is usually not in the alt_name.
    """
    if room["alt_name"] is None:
        return
    alt_parts = room["alt_name"].split(",")
    if len(alt_parts) < 2:
        return
    if alt_parts[0].lower() == arch_name_parts[0].lower():
        room["alt_name"] = ", ".join(alt_parts[1:]).strip()
        return
    # The most common mismatch is if the roomname in the alt_name is like "L516" and the arch_name starts with "L 516".
    # In this case we change the arch_name to the format without a space
    joined_roomname = arch_name_parts[0].replace(" ", "", 2)
    if arch_name_parts[0][:2] in {"R ", "L ", "M ", "N "} and alt_parts[0] == joined_roomname:
        arch_name_parts = alt_parts[0], arch_name_parts[1]
        room["arch_name"] = "@".join(arch_name_parts)
        room["alt_name"] = ", ".join(alt_parts[1:])
        room["patched"] = True
    # The same might appear the other way round (e.g. "N 1070 ZG" and "N1070ZG"),
    # or the roomname in the alt_name is prepended with the abbrev of the building (e.g. "MW 1050").
    elif (alt_parts[0][:2] in {"N ", "R "} and arch_name_parts[0] == alt_parts[0].replace(" ", "")) or any(
        alt_parts[0].startswith(s) for s in ["PH ", "MW ", "WSI ", "CH ", "MI "]
    ):
        room["alt_name"] = ", ".join(alt_parts[1:])
        room["patched"] = True
    # If the roomname has a comma, the comparison by parts fails
    elif "," in arch_name_parts[0] and room["alt_name"] is not None and room["alt_name"].startswith(arch_name_parts[0]):
        alt_name_index = arch_name_parts[0].count(",") + 1
        room["alt_name"] = ", ".join(alt_parts[alt_name_index:])
        room["patched"] = True
    # The Theresianum is an exception where the roomname is the second part of the alt_name.
    # Both are discarded since both roomcode and building name are given separately
    elif alt_parts[0] == "Theresianum" and alt_parts[1] == arch_name_parts[0]:
        room["alt_name"] = ", ".join(alt_parts[2:])
        room["patched"] = True
    else:
        _logger.debug(
            f"(alt_name / arch_name mismatch): {alt_parts[0]=} {arch_name_parts[0]=} {room_code=}",
        )
    if room["alt_name"] is not None:
        room["alt_name"] = room["alt_name"].strip()
