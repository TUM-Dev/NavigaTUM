import logging


def add_children_properties(data):
    """
    Add the "children" and "children_flat" properties to every object
    using the "parents" property.
    This operates on the data dict directly without creating a copy.
    """
    for _id, entry in data.items():
        for i, parent in enumerate(reversed(entry["parents"])):
            data[parent].setdefault("children_flat", []).append(_id)
            if i == 0:
                data[parent].setdefault("children", []).append(_id)


def add_stats(data):
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


def infer_addresses(data):
    """
    Infer addresses from children.
    """
    for _id, entry in data.items():
        if entry.get("props", {}).get("address", None) is None and "children_flat" in entry:
            child_addresses = set()

            for child_id in entry["children_flat"]:
                child = data[child_id]

                street, plz_place = (
                    child.get("props", {}).get("address", {}).get("street", None),
                    child.get("props", {}).get("address", {}).get("plz_place", None),
                )

                if street is not None and plz_place is not None:
                    child_addresses.add((street, plz_place))

            # If there are multiple, try removing the floor level
            if len(child_addresses) > 1:
                new_child_addresses = set()
                for street, plz_place in child_addresses:
                    new_child_addresses.add((",".join(street.split(",")[:-1]), plz_place))
                child_addresses = new_child_addresses

            if len(child_addresses) == 1:
                street, plz_place = child_addresses.pop()
                entry.setdefault("props")["address"] = {
                    "street": street,
                    "plz_place": plz_place,
                    "source": "inferred",
                }


def infer_type_common_name(data):
    """This function infers the type_common_name property for each entry via the type property."""
    for _id, _data in data.items():
        _data["type_common_name"] = {
            "root": "Standortübersicht",
            "site": "Standort",
            "campus": "Campus",
            "area": "Gebiet / Gruppe von Gebäuden",
            "joined_building": "Gebäudekomplex",
            "building": "Gebäudeteil"
            if (_data["type"] == "building" and data[_data["parents"][-1]]["type"] == "joined_building")
            else "Gebäude",
            "room": _data["usage"]["name"] if "usage" in _data else "Raum",
            "virtual_room": _data["usage"]["name"] if "usage" in _data else "Raum/Gebäudeteil",
        }[_data["type"]]
