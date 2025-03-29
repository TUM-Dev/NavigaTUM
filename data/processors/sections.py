import typing
from typing import Any

from utils import TranslatableStr

_ = TranslatableStr


def extract_tumonline_props(data: dict[str, dict[str, Any]]) -> None:
    """Extract some of the TUMonline data and provides it as `prop`."""
    for entry in data.values():
        if entry.get("tumonline_data", {}).get("calendar", None):
            calendar_resource_id = entry["tumonline_data"]["calendar"]
            calendar_url = f"https://campus.tum.de/tumonline/tvKalender.wSicht?cOrg=0&cRes={calendar_resource_id}"
            entry["props"]["calendar_url"] = calendar_url
        if entry.get("tumonline_data", {}).get("operator", None):
            entry["props"]["operator"] = {
                "code": entry["tumonline_data"]["operator"],
                "name": entry["tumonline_data"]["operator_name"],
                "url": (
                    f"https://campus.tum.de/tumonline/webnav.navigate_to?corg={entry['tumonline_data']['operator_id']}"
                ),
                "id": entry["tumonline_data"]["operator_id"],
            }
        if tumonline_id := entry.get("tumonline_data", {}).get("tumonline_id", None):
            entry["props"]["tumonline_room_nr"] = tumonline_id


def compute_floor_prop(data: dict[str, Any]) -> None:
    """
    Create a human and machine-readable floor information prop.

    This takes into account special floor numbering systems of buildings.
    """
    for _id, entry in data.items():
        if entry["type"] not in {"building", "joined_building"}:
            continue

        parent_type = data[entry["parents"][-1]]["type"]
        if parent_type == "joined_building" or "children_flat" not in entry:
            continue

        room_data = _collect_floors_room_data(data, entry)
        floor_details = _get_floor_details(entry, room_data)

        entry.setdefault("props", {})["floors"] = floor_details

        # Now add this floor information to all children
        lookup = {floor["tumonline"]: floor for floor in floor_details}
        for room in room_data:
            room_entry = data[room["id"]]
            room_entry.setdefault("props", {})["floor"] = lookup[room["floor"]]


def _collect_floors_room_data(data: dict[str, Any], entry: dict[str, Any]) -> list[dict[str, Any]]:
    """Collect floors of a (joined_)building"""
    room_data = []
    for child_id in entry["children_flat"]:
        child = data[child_id]
        if child["type"] == "room" and "ids" in child.get("props", {}):
            roomcode = child["props"]["ids"].get("roomcode", None)

            floor = child.get("generators", {}).get("floors", {}).get("floor_patch", roomcode.split(".")[1])

            room_data.append(
                {
                    "id": child_id,
                    "floor": floor,
                },
            )

    return room_data


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


def compute_props(data: dict[str, Any]) -> None:
    """Create the "computed" value in "props"."""
    for _id, entry in data.items():
        if props := entry.get("props"):
            computed = _gen_computed_props(_id, entry, props)

            # Reformat if required (just to have less verbosity in the code above)
            reformatted_computed: list[RawComputedProp | TranslatedComputedProp] = []
            computed_prop: RawComputedProp | dict[TranslatableStr, str]
            for computed_prop in computed:
                if "name" in computed_prop:
                    reformatted_computed.append(
                        {
                            "name": computed_prop["name"],
                            "text": computed_prop["text"],
                        }
                    )
                else:
                    reformatted_computed.append(
                        {
                            "name": next(iter(computed_prop.keys())),
                            "text": next(iter(computed_prop.values())),
                        }
                    )

            entry["props"]["computed"] = reformatted_computed


def _append_if_present(
    props: dict,
    computed_results: list[dict[TranslatableStr, TranslatableStr | str]],
    key: str,
    human_name: TranslatableStr,
) -> None:
    if key in props and props[key] is not None:
        computed_results.append({human_name: str(props[key])})


def _gen_computed_props(
    _id: str,
    entry: dict[str, str],
    props: dict,
) -> list[dict[TranslatableStr | str, TranslatableStr | str]]:
    computed: list[dict[TranslatableStr, TranslatableStr | str]] = []
    if "ids" in props:
        _append_if_present(props["ids"], computed, "b_id", _("Gebäudekennung"))
        _append_if_present(props["ids"], computed, "roomcode", _("Raumkennung"))
        if "arch_name" in props["ids"]:
            computed.append({_("Architekten-Name"): props["ids"]["arch_name"].split("@")[0]})
    if floor := props.get("floor"):
        if floor["trivial"]:
            computed.append({_("Stockwerk"): floor["name"]})
        else:
            computed.append({_("Stockwerk"): f"{floor['floor']} (" + floor["name"] + ")"})
    if "b_prefix" in entry and entry["b_prefix"] != _id:
        b_prefix = [entry["b_prefix"]] if isinstance(entry["b_prefix"], str) else entry["b_prefix"]
        building_names = ", ".join([p.ljust(4, "x") for p in b_prefix])
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
    return computed


def localize_links(data: dict[str, Any]) -> None:
    """
    Reformat the "links" value in "props" to be explicitly localized.

    This is a convenience function for the source data format that converts e.g.:
      `text: "<str>"`
    into
      `text: { de: "<str>", en: "<str>" }`
    """
    for entry in data.values():
        if links := entry.get("props", {}).get("links", None):
            for link in links:
                if isinstance(link["text"], str):
                    link["text"] = {"de": link["text"], "en": link["text"]}
                if isinstance(link["url"], str):
                    link["url"] = {"de": link["url"], "en": link["url"]}


def generate_buildings_overview(data: dict[str, Any]) -> None:
    """Generate the "buildings_overview" section"""
    for _id, entry in data.items():
        if entry["type"] not in {"area", "site", "campus"} or "children_flat" not in entry:
            continue

        options = entry.get("generators", {}).get("buildings_overview", {"n_visible": 6, "list_start": []})

        # Collect buildings to display for this entry.
        buildings = []
        for child_id in entry["children"]:
            child = data[child_id]
            if child["type"] in {"area", "site", "campus", "building", "joined_building"}:
                buildings.append(child)
        # for child_id in entry["children_flat"]:
        #    child = data[child_id]
        #    if child["type"] == "joined_building" or \
        #       (child["type"] == "building"
        #        and data[child["parents"][-1]]["type"] != "joined_building"):
        #        buildings.append(child)
        # Entries are sorted alphabetically in second order to be predictable
        buildings = sorted(buildings, key=lambda e: (len(e.get("children_flat", [])), e["name"]), reverse=True)

        # The "list_start" can overwrite how the list of buildings starts,
        # and optionally also add other entries. All other entries are appended
        # after them.
        merged_ids = options["list_start"] + [b["id"] for b in buildings if b["id"] not in options["list_start"]]

        b_overview = entry.setdefault("sections", {}).setdefault("buildings_overview", {})
        b_overview["n_visible"] = options["n_visible"]
        b_overview["entries"] = []
        for child_id in merged_ids:
            try:
                child = data[child_id]
            except KeyError as err:
                raise RuntimeError(f"Unknown id '{child_id}' when generating buildings_overview for '{_id}'") from err

            n_rooms = child["props"]["stats"].get("n_rooms", 0)
            n_buildings = child["props"]["stats"].get("n_buildings", 0)
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

            b_overview["entries"].append(
                {
                    "id": child_id,
                    "name": child["short_name"] if "short_name" in child else child["name"],
                    "subtext": subtext,
                    "thumb": child["imgs"][0]["name"] if child.get("imgs", []) else None,
                },
            )


def generate_rooms_overview(data: dict[str, dict[str, Any]]) -> None:
    """Generate the "rooms_overview" section"""
    for _id, entry in data.items():
        # if entry["type"] not in {"building", "joined_building", "virtual_room"} or \
        if (
            entry["type"] not in {"area", "site", "campus", "building", "joined_building", "virtual_room"}
            or "children_flat" not in entry
        ):
            continue

        rooms = {}
        for child_id in entry["children_flat"]:
            child = data[child_id]
            if child["type"] == "room":
                usage = child["usage"] if "usage" in child else {"name": _("Unbekannt")}
                rooms.setdefault(usage["name"], []).append(
                    {
                        "id": child_id,
                        "name": child["name"],
                    },
                )

        r_overview = entry.setdefault("sections", {}).setdefault("rooms_overview", {})
        r_overview["usages"] = [
            {
                "name": u[0],
                "count": len(u[1]),
                "children": sorted(u[1], key=lambda r: r["name"]),
            }
            for u in sorted(rooms.items(), key=lambda e: e[0])
        ]
