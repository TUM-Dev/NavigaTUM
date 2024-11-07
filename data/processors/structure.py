import logging
import typing
from typing import Any

from utils import TranslatableStr as _


def add_children_properties(data: dict[str, dict[str, Any]]) -> None:
    """
    Add the "children" and "children_flat" properties to every object using the "parents" property.

    This operates on the data dict directly without creating a copy.
    """
    for _id, entry in data.items():
        for i, parent in enumerate(reversed(entry["parents"])):
            data[parent].setdefault("children_flat", []).append(_id)
            if i == 0:
                data[parent].setdefault("children", []).append(_id)


def add_stats(data: dict[str, dict[str, Any]]) -> None:
    """
    Calculate structural statistics for each entry (number of children etc).

    This requires the children property.
    """
    for _id, entry in data.items():
        stats = entry.setdefault("props", {}).setdefault("stats", {})

        if "children_flat" not in entry:
            if entry["type"] in {"root", "site", "campus", "area"}:
                logging.warning(f"'{_id}' ({entry['type']}) has no children")
            continue

        n_rooms = 0
        n_rooms_reg = 0
        n_buildings = 0

        for child_id in entry["children_flat"]:
            child = data[child_id]

            # Note: "virtual_rooms" are not counted as rooms, since they are more of
            # an indoor space, but may consist of several real rooms
            if child["type"] == "room":
                n_rooms += 1
                if not child.get("usage", {}).get("din_277", "").startswith("VF"):
                    n_rooms_reg += 1
            if child["type"] == "joined_building" or (
                child["type"] == "building" and data[child["parents"][-1]]["type"] != "joined_building"
            ):
                n_buildings += 1

        if entry["type"] in {"root", "site", "campus", "area", "joined_building", "building"}:
            stats["n_rooms"] = n_rooms
            stats["n_rooms_reg"] = n_rooms_reg
            if n_rooms == 0:
                logging.warning(f"'{_id}' ({entry['type']}) has no rooms")
        if entry["type"] in {"root", "site", "campus", "area"}:
            stats["n_buildings"] = n_buildings


class AddressTuple(typing.NamedTuple):
    street: str
    plz_place: str


def infer_addresses(data: dict[str, dict[str, Any]]) -> None:
    """Infer addresses from children."""
    for _id, entry in data.items():
        if entry.get("props", {}).get("address", None) is None and (children_flat := entry.get("children_flat")):
            child_addresses: set[AddressTuple] = set()

            for child_id in children_flat:
                child = data[child_id]

                address = child.get("props", {}).get("address", {})
                street: str | None = address.get("street", None)
                plz_place: str | None = address.get("plz_place", None)
                if (zip_code := address.get("zip_code", None)) and (place := address.get("place", None)):
                    plz_place = f"{zip_code} {place}"

                if street is not None and plz_place is not None:
                    child_addresses.add(AddressTuple(street, plz_place))

            if len(child_addresses) == 1:
                address = child_addresses.pop()
                entry.setdefault("props").setdefault(
                    "address",
                    {
                        "street": address.street,
                        "plz_place": address.plz_place,
                        "source": "inferred",
                    },
                )


TYPE_COMMON_NAME_BY_TYPE = {
    "root": _("Standortübersicht"),
    "site": _("Standort"),
    "campus": "Campus",
    "area": _("Gebiet / Gruppe von Gebäuden"),
    "joined_building": _("Gebäudekomplex"),
    "building": _("Gebäude"),
    "room": _("Raum"),
    "virtual_room": _("Raum/Gebäudeteil"),
    "poi": "POI",
}


def infer_type_common_name(data: dict[str, dict[str, Any]]) -> None:
    """Infer the type_common_name property for each entry via the type property."""
    for _data in data.values():
        building_inside_joined_building = (
            _data["type"] == "building" and data[_data["parents"][-1]]["type"] == "joined_building"
        )
        if building_inside_joined_building:
            _data["type_common_name"] = _("Gebäudeteil")
        elif _data["type"] in {"room", "virtual_room", "poi"} and "usage" in _data:
            _data["type_common_name"] = _data["usage"]["name"]
        else:
            _data["type_common_name"] = TYPE_COMMON_NAME_BY_TYPE[_data["type"]]
