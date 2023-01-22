import json
import logging
from collections import Counter
from dataclasses import dataclass


@dataclass
class NATBuilding:
    b_id: str
    b_name: str
    b_tumonline_id: None | int
    b_alias: None | str
    b_address: None | str

    def __init__(self, data: dict):
        self.b_id = data["building_code"]
        self.b_name = data["building_name"]
        self.b_tumonline_id = data["building_id"]
        self.b_alias = data["building_short"]
        self.b_address = data["address"]


def merge_nat_buildings(data):
    """
    Merge the buildings in the NAT Roomfinder with the existing data.
    This will overwrite existing data, as they have patched some fields.
    """
    with open("external/results/buildings_nat.json", encoding="utf-8") as file:
        buildings = json.load(file)

    # Sanity-check: Make sure that the buildings in the data are unique
    building_ids = [b["building_code"] for b in buildings]
    duplicate_building_ids = {b_id: cnt for b_id, cnt in Counter(building_ids).items() if cnt > 1}
    if duplicate_building_ids:
        raise ValueError(f"There are duplicate buildings in the data: {duplicate_building_ids}")

    for building in [NATBuilding(b) for b in buildings]:
        if building.b_id in [
            "0000",  # 'Building' 0000 contains some buildings and places not in TUMonline as rooms.
            # They might be integrated custom somewhere else, but here we ignore these.
            "3002",  # "Testgebäude 2" => building which probably does not exist
            "5110",  # wurde Abgerissen
            "5537",  # "Interims-Zelt-TUMshop" => building which no longer exists
            "0598", "4298", "5538", "5998",  # "Interims-Tentomax => buildings no longer exist
            "5516", "5600",  # phantom buildings, which don't exist
            "XXXX",  # "virtueller Raum"
        ]:
            continue

        internal_id = _infer_internal_id(building, data)
        _merge_building(data, internal_id, building)


def _infer_internal_id(building, data):
    if building.b_id.startswith("X"):
        # they have external buildings in there (for example Max-Planck-Institut für Plasmaphysik)
        # we added them to the roomfinder data, but did not keept thier scheme
        building.b_id = building.b_id[1:].lower()
        return building.b_id

    internal_id = None
    for _id, _data in data.items():
        if "b_prefix" in _data and _data["b_prefix"] == building.b_id:
            if internal_id is not None:
                raise RuntimeError(f"building id '{building.b_id}' more than once in base data")
            internal_id = _id
    if internal_id is None:
        raise RuntimeError(f"building '{building}' not found in base data. "
                           f"It may be missing in the areatree.")
    return internal_id


def _merge_building(data, internal_id, building):
    used_as_source = internal_id in data
    b_data = data[internal_id]
    b_data["nat_data"] = building

    # Data fixes
    used_as_source |= b_data["name"] == building.b_name
    b_data["name"] = building.b_name

    b_data["name"] = building.b_name

    if used_as_source:
        b_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "NAT Building Data",
                "url": f"https://www.ph.tum.de/about/visit/roomfinder/?room={building.b_id}",
            },
        )
    b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", building.b_id)


def merge_nat_rooms(data):
    """
    Merge the rooms in the NAT Roomfinder with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """

    with open("external/results/rooms_nat.json", encoding="utf-8") as file:
        rooms = json.load(file)

    # TODO: implement the merging of NAT rooms
    logging.warning("Merging NAT rooms is not yet implemented")