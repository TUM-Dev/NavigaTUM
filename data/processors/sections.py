import json
import logging
import typing
from typing import Any

import polars as pl

from utils import TranslatableStr

_ = TranslatableStr


def extract_tumonline_props(lf: pl.LazyFrame) -> pl.LazyFrame:
    """Extract some of the TUMonline data and provides it as prop columns."""

    def _json_not_null(field: str) -> pl.Expr:
        extracted = pl.col("tumonline_data_json").str.json_path_match(f"$.{field}")
        return extracted.is_not_null() & (extracted != "null")

    lf = lf.with_columns(
        [
            # calendar_url
            pl.when(_json_not_null("calendar"))
            .then(
                pl.lit("https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=0&cRes=")
                + pl.col("tumonline_data_json").str.json_path_match("$.calendar")
            )
            .otherwise(pl.col("props_calendar_url"))
            .alias("props_calendar_url"),
            # operator code
            pl.when(_json_not_null("operator"))
            .then(pl.col("tumonline_data_json").str.json_path_match("$.operator"))
            .otherwise(pl.col("props_operator_code"))
            .alias("props_operator_code"),
            # operator_url
            pl.when(_json_not_null("operator_id"))
            .then(
                pl.lit("https://campus.tum.de/tumonline/webnav.navigate_to?corg=")
                + pl.col("tumonline_data_json").str.json_path_match("$.operator_id")
            )
            .otherwise(pl.col("props_operator_url"))
            .alias("props_operator_url"),
            # operator_id
            pl.when(_json_not_null("operator_id"))
            .then(pl.col("tumonline_data_json").str.json_path_match("$.operator_id").cast(pl.Int64))
            .otherwise(pl.col("props_operator_id"))
            .alias("props_operator_id"),
            # tumonline_room_nr
            pl.when(_json_not_null("tumonline_id"))
            .then(pl.col("tumonline_data_json").str.json_path_match("$.tumonline_id").cast(pl.Int64))
            .otherwise(pl.col("props_tumonline_room_nr"))
            .alias("props_tumonline_room_nr"),
        ]
    )

    # operator_name (de/en)
    lf = lf.with_columns(
        [
            pl.when(_json_not_null("operator_name"))
            .then(pl.col("tumonline_data_json").str.json_path_match("$.operator_name.de"))
            .otherwise(pl.col("props_operator_name_de"))
            .alias("props_operator_name_de"),
            pl.when(_json_not_null("operator_name"))
            .then(pl.col("tumonline_data_json").str.json_path_match("$.operator_name.en"))
            .otherwise(pl.col("props_operator_name_en"))
            .alias("props_operator_name_en"),
        ]
    )

    return lf


def compute_floor_prop(df: pl.DataFrame) -> pl.DataFrame:
    """
    Create a human and machine-readable floor information prop.

    This takes into account special floor numbering systems of buildings.
    """
    data_dicts = {
        row["id"]: row
        for row in df.select("id", "type", "children_flat", "props_ids_roomcode", "generators_json").to_dicts()
    }

    floor_updates: dict[str, str] = {}  # id -> props_floors_json

    for _id, entry in data_dicts.items():
        if entry["type"] not in {"building", "joined_building", "site", "campus"}:
            continue
        if not entry.get("children_flat"):
            logging.warning(f"Entry {_id} has no children")
            continue

        room_data = []
        for child_id in entry["children_flat"]:
            child = data_dicts.get(child_id, {})
            if child.get("type") == "room" and child.get("props_ids_roomcode"):
                generators = json.loads(child.get("generators_json") or "{}") if child.get("generators_json") else {}
                roomcode = child["props_ids_roomcode"]
                floor = generators.get("floors", {}).get("floor_patch", roomcode.split(".")[1])
                room_data.append({"id": child_id, "floor": floor})

        if not room_data:
            continue

        generators = json.loads(entry.get("generators_json") or "{}") if entry.get("generators_json") else {}
        mock_entry = {"generators": generators}
        floor_details = _get_floor_details(mock_entry, room_data)

        floor_updates[_id] = json.dumps(floor_details)

        lookup = {f["tumonline"]: f for f in floor_details}
        for room in room_data:
            floor_updates[room["id"]] = json.dumps([lookup[room["floor"]]])

    if floor_updates:
        updates_df = pl.DataFrame([{"id": k, "props_floors_json_new": v} for k, v in floor_updates.items()])
        df = df.join(updates_df, on="id", how="left")
        df = df.with_columns(
            pl.coalesce(pl.col("props_floors_json_new"), pl.col("props_floors_json")).alias("props_floors_json"),
        ).drop("props_floors_json_new")

    return df


def _build_sorted_floor_list(room_data):
    """Build a physically sorted list of floors (using TUMonline floor names)"""
    floors = {room["floor"] for room in room_data}

    def floor_quantifier(floor_name: str) -> int:
        """Assign each floor a virtual ID for sorting"""
        if floor_name == "EG":
            return 0
        if floor_name == "DG":
            return 1000
        if floor_name.startswith("U"):
            return -10 * int(floor_name[1:])
        if floor_name.isnumeric():
            return 10 * int(floor_name)
        if floor_name.startswith("Z"):
            # Default placement: Z1 is below 01 etc.
            return 10 * int(floor_name[1:]) - 5
        if floor_name == "TP":  # Tiefparterre / Semi-Basement
            # Default placement: below EG
            return -5
        raise RuntimeError(f"Unknown TUMonline floor name {floor_name}")

    return sorted(floors, key=floor_quantifier)


def _get_floor_details(entry, room_data):
    """Infer for each floor the metadata and name string"""
    floors = _build_sorted_floor_list(room_data)
    floors_details = []

    patches = entry.get("generators", {}).get("floors", {}).get("floor_patches", {})

    eg_index = floors.index("EG") if "EG" in floors else 0
    mezzanine_shift = 0
    for i, floor_tumonline in enumerate(floors):
        floor = patches.get(floor_tumonline, {}).get("use_as", floor_tumonline)
        f_id = patches.get(floor_tumonline, {}).get("id", i - eg_index)

        floor_type, floor_abbr, floor_name = _get_floor_name_and_type(f_id, floor, mezzanine_shift)

        # In trivial cases (e.g. "1 (1st upper floor)"), the information of floor_abbr and
        # floor_name is redundant, so we can get simplify the floor information.
        trivial = True
        if "name" in patches.get(floor_tumonline, {}):
            floor_name = patches[floor_tumonline]["name"]
            trivial = False
        elif floor_type in {"roof", "tp"} or mezzanine_shift > 0:
            trivial = False

        floors_details.append(
            {
                "id": f_id,
                "floor": floor_abbr,
                "tumonline": floor_tumonline,
                "type": floor_type,
                "name": floor_name,
                "mezzanine_shift": mezzanine_shift,
                "trivial": trivial,
            },
        )
        if i - eg_index >= 0 and floor.startswith("Z"):
            mezzanine_shift += 1

    return floors_details


def _get_floor_name_and_type(f_id: int, floor: str, mezzanine_shift: int) -> tuple[str, str, _]:
    """
    Generate a machine-readable floor type and human-readable floor name (long & short)

    :param f_id: Floor id (0 for ground floor if there is one, else 0 for the lowest)
    :param floor: Floor name in TUMonline
    :param mezzanine_shift: How many mezzanines are between this floor and floor 0 (only >= 0)
    :returns: A tuple of three elements:
              - The type name of the floor (ground | roof | tp | basement | mezzanine | upper)
              - A short string about the floor (e.g. "-1", "0", "Z1", "5")
              - A long TranslatableStr about the floor (e.g. "Erdgeschoss")
    """
    match floor:
        case "EG":
            if f_id != 0:
                raise RuntimeError(f"Floor id {f_id} for ground floor {floor} is not 0!")
            return "ground", "0", _("Erdgeschoss")
        case "DG":
            return "roof", str(f_id), _("Dachgeschoss")
        case "TP":
            return "tp", "TP", _("Tiefparterre")
        case _ if floor.startswith("U"):
            floor_name = _(f"{floor[1:]}. ") + _("Untergeschoss")
            return "basement", f"-{floor[1:]}", floor_name
        case floor if floor.startswith("Z"):
            if f_id == 1:
                floor_name = _("1. Zwischengeschoss, über EG")
            else:
                floor_name = _(f"{floor[1:]}. ") + _("Zwischengeschoss")
            return "mezzanine", floor, floor_name
    # default case, but mypy doesn't recognize `case _:`
    og_floor = int(floor[1:])
    match mezzanine_shift:
        case 0:
            floor_name = _(f"{og_floor}. ") + _("Obergeschoss")
        case 1:
            floor_name = _(f"{og_floor}. ") + _("OG + 1 Zwischengeschoss")
        case mezzanine_shift:
            floor_name = _(f"{og_floor}. ") + _("OG + {m} Zwischengeschosse").format(m=mezzanine_shift)
    return "upper", str(og_floor), floor_name


class RawComputedProp(typing.TypedDict):
    name: str
    text: str


class TranslatedComputedProp(typing.TypedDict):
    name: TranslatableStr
    text: TranslatableStr


def compute_props(df: pl.DataFrame) -> pl.DataFrame:
    """Create the "computed" value in "props" as props_computed_json column."""
    data_dicts = df.to_dicts()
    computed_updates: dict[str, str] = {}  # id -> props_computed_json

    for row in data_dicts:
        _id = row["id"]
        # Reconstruct a props dict from the flat columns
        props = _reconstruct_props(row)
        computed = _gen_computed_props(_id, row, props) if props else []

        # Reformat if required (just to have less verbosity in the code above)
        reformatted_computed: list[RawComputedProp | TranslatedComputedProp] = []
        for computed_prop in computed:
            if "name" in computed_prop:
                reformatted_computed.append(
                    {  # type: ignore[arg-type,misc]
                        "name": computed_prop["name"],
                        "text": computed_prop["text"],
                    }
                )
            else:
                reformatted_computed.append(
                    {  # type: ignore[arg-type,misc]
                        "name": next(iter(computed_prop.keys())),
                        "text": next(iter(computed_prop.values())),
                    }
                )

        computed_updates[_id] = json.dumps(reformatted_computed)

    if computed_updates:
        updates_df = pl.DataFrame([{"id": k, "props_computed_json_new": v} for k, v in computed_updates.items()])
        df = df.join(updates_df, on="id", how="left")
        df = df.with_columns(
            pl.coalesce(pl.col("props_computed_json_new"), pl.col("props_computed_json")).alias("props_computed_json"),
        ).drop("props_computed_json_new")

    return df


def _reconstruct_props(row: dict[str, Any]) -> dict[str, Any]:
    """Reconstruct a nested props dict from flat DataFrame columns for use by helper functions."""
    props: dict[str, Any] = {}

    # ids
    ids = {}
    if row.get("props_ids_b_id"):
        ids["b_id"] = row["props_ids_b_id"]
    if row.get("props_ids_roomcode"):
        ids["roomcode"] = row["props_ids_roomcode"]
    if row.get("props_ids_arch_name"):
        ids["arch_name"] = row["props_ids_arch_name"]
    if ids:
        props["ids"] = ids

    # floors
    if row.get("props_floors_json"):
        props["floors"] = json.loads(row["props_floors_json"])

    # address
    if row.get("props_address_street") or row.get("props_address_plz_place"):
        props["address"] = {
            "street": row.get("props_address_street", ""),
            "plz_place": row.get("props_address_plz_place", ""),
        }

    # stats
    stats = {}
    if row.get("props_stats_n_buildings") is not None:
        stats["n_buildings"] = row["props_stats_n_buildings"]
    if row.get("props_stats_n_rooms") is not None:
        stats["n_rooms"] = row["props_stats_n_rooms"]
    if row.get("props_stats_n_rooms_reg") is not None:
        stats["n_rooms_reg"] = row["props_stats_n_rooms_reg"]
    if row.get("props_stats_n_seats") is not None:
        stats["n_seats"] = row["props_stats_n_seats"]
    if stats:
        props["stats"] = stats

    # generic
    if row.get("props_generic_json"):
        props["generic"] = json.loads(row["props_generic_json"])

    # links
    if row.get("props_links_json"):
        props["links"] = json.loads(row["props_links_json"])

    return props


def _append_if_present(
    props: dict[str, Any],
    computed_results: list[dict[TranslatableStr, TranslatableStr | str]],
    key: str,
    human_name: TranslatableStr,
) -> None:
    if key in props and props[key] is not None:
        computed_results.append({human_name: str(props[key])})


def _gen_computed_props(
    _id: str,
    entry: dict[str, str],
    props: dict[str, Any],
) -> list[dict[TranslatableStr | str, TranslatableStr | str]]:
    computed: list[dict[TranslatableStr, TranslatableStr | str]] = []
    if "ids" in props:
        _append_if_present(props["ids"], computed, "b_id", _("Gebäudekennung"))
        _append_if_present(props["ids"], computed, "roomcode", _("Raumkennung"))
        if "arch_name" in props["ids"]:
            computed.append({_("Architekten-Name"): props["ids"]["arch_name"].split("@")[0]})
    if floors := props.get("floors"):
        if len(floors) == 1:
            floor = floors[0]
            floor_name = floor["name"]
            # floor_name may be a dict {de:..., en:...} from JSON deserialization
            if isinstance(floor_name, dict):
                floor_name = TranslatableStr(floor_name["de"], floor_name.get("en"))
            if floor["trivial"]:
                computed.append({_("Stockwerk"): floor_name})
            else:
                computed.append({_("Stockwerk"): f"{floor['floor']} (" + floor_name + ")"})
    b_prefix_raw: Any = entry.get("b_prefix_list") or entry.get("b_prefix")
    if b_prefix_raw and b_prefix_raw != _id:
        if isinstance(b_prefix_raw, list):
            b_prefix_vals = b_prefix_raw
        elif isinstance(b_prefix_raw, str):
            b_prefix_vals = [b_prefix_raw]
        else:
            b_prefix_vals = [str(b_prefix_raw)]
        building_names = ", ".join([p.ljust(4, "x") for p in b_prefix_vals])
        computed.append({_("Gebäudekennungen"): building_names})
    if address := props.get("address"):
        computed.append({_("Adresse"): f"{address['street']}, {address['plz_place']}"})
    if stats := props.get("stats"):
        _append_if_present(stats, computed, "n_buildings", _("Anzahl Gebäude"))
        _append_if_present(stats, computed, "n_seats", _("Sitzplätze"))
        if "n_rooms" in stats:
            if stats["n_rooms"] == stats["n_rooms_reg"]:
                computed.append({_("Anzahl Räume"): str(stats["n_rooms"])})
            else:
                value = _("{n_rooms} ({n_rooms_reg} ohne Flure etc.)").format(
                    n_rooms=stats["n_rooms"],
                    n_rooms_reg=stats["n_rooms_reg"],
                )
                computed.append({_("Anzahl Räume"): value})
    if generic_props := props.get("generic"):
        computed.extend(generic_props)
    return computed  # type: ignore[return-value]


def localize_links(df: pl.DataFrame) -> pl.DataFrame:
    """
    Reformat the "links" value in "props" to be explicitly localized.

    This is a convenience function for the source data format that converts e.g.:
      `text: "<str>"`
    into
      `text: { de: "<str>", en: "<str>" }`
    """

    def _localize_links_json(links_json: str | None) -> str | None:
        if links_json is None:
            return None
        links = json.loads(links_json)
        if not links:
            return links_json
        for link in links:
            if isinstance(link["text"], str):
                link["text"] = {"de": link["text"], "en": link["text"]}
            if isinstance(link["url"], str):
                link["url"] = {"de": link["url"], "en": link["url"]}
        return json.dumps(links)

    df = df.with_columns(
        pl.col("props_links_json").map_elements(_localize_links_json, return_dtype=pl.Utf8).alias("props_links_json"),
    )
    return df


def generate_buildings_overview(df: pl.DataFrame) -> pl.DataFrame:
    """Generate the "buildings_overview" section."""
    # Build lookup dict from DataFrame for child access
    data_dicts = {row["id"]: row for row in df.to_dicts()}

    overview_updates: dict[str, str] = {}  # id -> sections_buildings_overview_json

    for _id, entry in data_dicts.items():
        if entry["type"] not in {"area", "site", "campus"} or not entry.get("children_flat"):
            continue

        generators = json.loads(entry.get("generators_json") or "{}") if entry.get("generators_json") else {}
        options = generators.get("buildings_overview", {"n_visible": 6, "list_start": []})

        # Collect buildings to display for this entry.
        children = entry.get("children") or []
        if isinstance(children, str):
            children = json.loads(children)

        buildings = []
        for child_id in children:
            child = data_dicts.get(child_id)
            if child and child["type"] in {"area", "site", "campus", "building", "joined_building"}:
                buildings.append(child)

        # Entries are sorted alphabetically in second order to be predictable
        buildings = sorted(
            buildings,
            key=lambda e: (len(e.get("children_flat") or []), e["name"]),
            reverse=True,
        )

        # The "list_start" can overwrite how the list of buildings starts
        merged_ids = options["list_start"] + [b["id"] for b in buildings if b["id"] not in options["list_start"]]

        b_overview: dict[str, Any] = {
            "n_visible": options["n_visible"],
            "entries": [],
        }
        for child_id in merged_ids:
            child = data_dicts.get(child_id)
            if child is None:
                raise RuntimeError(f"Unknown id '{child_id}' when generating buildings_overview for '{_id}'")

            # Reconstruct stats from flat columns
            n_rooms = child.get("props_stats_n_rooms") or 0
            n_buildings = child.get("props_stats_n_buildings") or 0

            if child["type"] in {"building", "joined_building"}:
                if n_rooms == 0:
                    subtext = _("Keine Räume bekannt")
                else:
                    subtext = _("{n_rooms} Räume").format(n_rooms=n_rooms)
            elif child["type"] == "area":
                subtext = _("{n_buildings} Gebäude, {n_rooms} Räume").format(n_buildings=n_buildings, n_rooms=n_rooms)
            elif child["type"] == "site":
                subtext = _("{n_buildings} Gebäude, {n_rooms} Räume (Außenstelle)").format(
                    n_buildings=n_buildings,
                    n_rooms=n_rooms,
                )
            else:
                raise RuntimeError(
                    f"Cannot generate buildings_overview subtext for type '{child['type']}', "
                    f"for: '{_id}', child id: '{child_id}'",
                )

            # Get first image thumb name
            imgs = json.loads(child.get("imgs_json") or "[]") if child.get("imgs_json") else []
            thumb = imgs[0]["name"] if imgs else None

            b_overview["entries"].append(
                {
                    "id": child_id,
                    "name": child.get("short_name") or child["name"],
                    "subtext": subtext,
                    "thumb": thumb,
                },
            )

        overview_updates[_id] = json.dumps(b_overview)

    if overview_updates:
        updates_df = pl.DataFrame(
            [{"id": k, "sections_buildings_overview_json_new": v} for k, v in overview_updates.items()]
        )
        df = df.join(updates_df, on="id", how="left")
        df = df.with_columns(
            pl.coalesce(
                pl.col("sections_buildings_overview_json_new"),
                pl.col("sections_buildings_overview_json"),
            ).alias("sections_buildings_overview_json"),
        ).drop("sections_buildings_overview_json_new")

    return df


def generate_rooms_overview(df: pl.DataFrame) -> pl.DataFrame:
    """Generate the "rooms_overview" section."""
    data_dicts = {row["id"]: row for row in df.to_dicts()}

    overview_updates: dict[str, str] = {}  # id -> sections_rooms_overview_json

    for _id, entry in data_dicts.items():
        if entry["type"] not in {
            "area",
            "site",
            "campus",
            "building",
            "joined_building",
            "virtual_room",
        } or not entry.get("children_flat"):
            continue

        rooms: dict[TranslatableStr, list[dict[str, str]]] = {}
        for child_id in entry["children_flat"]:
            child = data_dicts.get(child_id)
            if child is None or child["type"] != "room":
                continue

            # Reconstruct usage from flat columns
            if child.get("usage_name_de"):
                usage_name = TranslatableStr(child["usage_name_de"], child.get("usage_name_en"))
            else:
                usage_name = _("Unbekannt")

            rooms.setdefault(usage_name, []).append(
                {
                    "id": child_id,
                    "name": child["name"],
                },
            )

        r_overview = {
            "usages": [
                {
                    "name": u_name,
                    "count": len(u_rooms),
                    "children": sorted(u_rooms, key=lambda r: r["name"]),
                }
                for u_name, u_rooms in sorted(rooms.items(), key=lambda e: e[0])
            ],
        }

        overview_updates[_id] = json.dumps(r_overview)

    if overview_updates:
        updates_df = pl.DataFrame(
            [{"id": k, "sections_rooms_overview_json_new": v} for k, v in overview_updates.items()]
        )
        df = df.join(updates_df, on="id", how="left")
        df = df.with_columns(
            pl.coalesce(
                pl.col("sections_rooms_overview_json_new"),
                pl.col("sections_rooms_overview_json"),
            ).alias("sections_rooms_overview_json"),
        ).drop("sections_rooms_overview_json_new")

    return df
