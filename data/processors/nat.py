import dataclasses
import logging
from collections import Counter
from pathlib import Path
from typing import Any

import yaml
from external.models import nat

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"

with (SOURCES_PATH / "12_nat_excluded_buildings.yaml").open(encoding="utf-8") as excluded_buildings_raw:
    EXCLUDED_BUILDINGS = set(yaml.safe_load(excluded_buildings_raw.read()))


@dataclasses.dataclass
class NATBuilding:
    b_id: None | str
    b_code: str
    b_name: str
    b_tumonline_id: None | int
    b_alias: None | str
    b_address: None | str

    def __init__(self, data: nat.Building):
        self.b_id = None  # Later set by _infer_internal_id()
        self.b_code = data.building_code  # Building id/code used by the NAT roomfinder
        self.b_name = data.building_name
        self.b_tumonline_id = data.building_id
        self.b_alias = data.building_short
        self.b_address = data.address


def merge_nat_buildings(data: dict[str, dict[str, Any]]) -> None:
    """
    Merge the buildings in the NAT Roomfinder with the existing data.

    This may overwrite existing data, if they have patched some fields.
    """
    buildings = nat.Building.load_all()

    # Sanity-check: Make sure that the buildings in the data are unique
    building_ids = [b.building_code for b in buildings]
    if duplicate_building_ids := {b_id: cnt for b_id, cnt in Counter(building_ids).items() if cnt > 1}:
        raise ValueError(f"There are duplicate buildings in the data: {duplicate_building_ids}")

    for building in [NATBuilding(b) for b in buildings]:
        if building.b_code in EXCLUDED_BUILDINGS:
            continue

        _merge_building(data, building)


def _infer_internal_id(building, data):
    # The NAT Roomfinder has buildings in it, that are not in TUMonline
    # (for example Max-Planck-Institut für Plasmaphysik). We keep them,
    # but use a different building id.
    if building.b_code.startswith("X"):
        if building.b_code == "XUCL":
            building.b_id = "origins-cluster"
        elif building.b_code == "XMPG":
            building.b_id = "mpi"
        else:
            building.b_id = building.b_code[1:].lower()

        return building.b_id

    for _id, _data in data.items():
        if "b_prefix" in _data and _data["b_prefix"] == building.b_code:
            if building.b_id is not None:
                raise RuntimeError(f"Building id '{building.b_code}' more than once in base data")
            building.b_id = _id
    if building.b_id is None:
        raise RuntimeError(
            f"Building '{building}' not found in base data. It may be missing in the areatree.",
        )
    return building.b_id


def _merge_building(data: dict, building: NATBuilding) -> None:
    internal_id = _infer_internal_id(building, data)

    b_data = data[internal_id]
    b_data["nat_data"] = dataclasses.asdict(building)

    # NAT buildings are merged after TUMonline and the MyTUM Roomfinder. So if the others
    # weren't used as sources, but the NAT Roomfinder has this building, we know it's from there.
    base_sources = b_data.setdefault("sources", {}).setdefault("base", [])
    if not base_sources:
        base_sources.append(
            {
                "name": "NAT Roomfinder",
                "url": f"https://www.ph.tum.de/about/visit/roomfinder/?room={building.b_code}",
            },
        )
    b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", building.b_id)


def merge_nat_rooms(_data):
    """
    Merge the rooms in the NAT Roomfinder with the existing data.

    This will not overwrite the existing data, but act directly on the provided data.
    """
    _rooms = nat.Room.load_all()

    # TODO: implement the merging of NAT rooms
    logging.warning("Merging NAT rooms is not yet implemented")
