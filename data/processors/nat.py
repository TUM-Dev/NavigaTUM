import json
import logging
from collections import Counter
from dataclasses import dataclass

import yaml
from utils import TranslatableStr as _


def load_excluded_buildings():
    """Load excluded buildings from config (own function so its variables are scoped)"""
    with open("sources/12_nat_excluded_buildings.yaml", encoding="utf-8") as file:
        return set(yaml.safe_load(file.read()))


EXCLUDED_BUILDINGS = load_excluded_buildings()


@dataclass
class NATBuilding:
    b_code: str
    b_name: str
    b_tumonline_id: None | int
    b_alias: None | str
    b_address: None | str

    def __init__(self, data: dict):
        self.b_code = data["building_code"]  # Building id/code used by the NAT roomfinder
        self.b_name = data["building_name"]
        self.b_tumonline_id = data["building_id"]
        self.b_alias = data["building_short"]
        self.b_address = data["address"]

    def as_dict(self):
        """Return the building data as dict"""
        return self.__dict__


def merge_nat_buildings(data):
    """
    Merge the buildings in the NAT Roomfinder with the existing data.
    This may overwrite existing data, if they have patched some fields.
    """
    with open("external/results/buildings_nat.json", encoding="utf-8") as file:
        buildings = json.load(file)

    # Sanity-check: Make sure that the buildings in the data are unique
    building_ids = [b["building_code"] for b in buildings]
    duplicate_building_ids = {b_id: cnt for b_id, cnt in Counter(building_ids).items() if cnt > 1}
    if duplicate_building_ids:
        raise ValueError(f"There are duplicate buildings in the data: {duplicate_building_ids}")

    for building in [NATBuilding(b) for b in buildings]:
        if building.b_code in EXCLUDED_BUILDINGS:
            continue

        _merge_building(data, building)


def _infer_internal_id(b_code, data):
    # The NAT Roomfinder has buildings in it, that are not in TUMonline
    # (for example Max-Planck-Institut für Plasmaphysik). We keep them,
    # but use a different building id.
    if b_code.startswith("X"):
        if b_code == "XUCL":
            return "origins-cluster"
        return b_code[1:].lower()

    if b_code in data:
        return b_code

    raise RuntimeError(
        f"Building id '{b_code}' not found in base data. It may be missing in the areatree.",
    )


def _merge_building(data, building: NATBuilding) -> None:
    internal_id = _infer_internal_id(building.b_code, data)

    b_data = data[internal_id]
    b_data["nat_data"] = building.as_dict()

    # NAT buildings are merged after TUMonline and the MyTUM Roomfinder. So if the others
    # weren't used as sources, but the NAT Roomfinder has this building, we know it's from there.
    # All buildings are at least in the areatree, which is always the first source.
    base_sources = b_data.setdefault("sources", {}).setdefault("base", [])
    if len(base_sources) == 1:
        base_sources.append(
            {
                "name": "NAT Roomfinder",
                "url": f"https://www.ph.tum.de/about/visit/roomfinder/?room={building.b_code}",
            },
        )
    b_data.setdefault("props", {}).setdefault("ids", {}).setdefault("b_id", internal_id)


def merge_nat_rooms(_data):
    """
    Merge the rooms in the NAT Roomfinder with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """

    with open("external/results/rooms_nat.json", encoding="utf-8") as file:
        _rooms = json.load(file)

    with open("external/results/usages_tumonline.json", encoding="utf-8") as file:
        usages = json.load(file)

    not_merged_parent = 0
    not_merged_outdated = 0
    for nat_id, nat_data in rooms.items():
        b_code, id_rest = nat_id.split(".", 1)

        if b_code in EXCLUDED_BUILDINGS:
            not_merged_parent += 1
            continue

        b_id = _infer_internal_id(b_code, data)
        internal_id = b_id + "." + id_rest

        if _is_room_excluded(internal_id, b_id, data):
            not_merged_outdated += 1
            continue

        r_data = _get_room_base(internal_id, b_id, nat_data, data, usages)

        r_data.setdefault("sources", {}).setdefault("base", []).append(
            {
                "name": "NAT Roomfinder",
                "url": f"https://www.ph.tum.de/about/visit/roomfinder/?room={nat_id}",
            },
        )

        _merge_nat_extra_props(r_data, nat_data)

    logging.debug(
        f"{not_merged_parent} rooms not merged because their parent buildings were not merged.",
    )
    logging.debug(
        f"{not_merged_outdated} rooms not merged because their buildings "
        f"are not exclusively from the NAT roomfinder (possibly outdated data).",
    )


def _is_room_excluded(internal_id, b_id, data):
    if internal_id in data:
        return False

    building_sources = data[b_id].get("sources").get("base")
    # First source for buildings is always the areatree, so we're checking the second one.
    if len(building_sources) == 2 and building_sources[1]["name"] == "NAT Roomfinder":
        return False

    return True


def _get_room_base(internal_id, b_id, nat_data, data, usages):
    if internal_id in data:
        return data[internal_id]

    if nat_data["number"]:
        room_alt_name = nat_data["description"].replace(nat_data["number"], "").lstrip(",").strip()
    else:
        room_alt_name = nat_data["description"]

    tumonline_usage = None
    for usage in usages:
        if usage["id"] == nat_data["purpose_id"]:
            tumonline_usage = usage
            break

    if tumonline_usage is None:
        raise RuntimeError(
            f"Can't find TUMonline usage data for usage id '{nat_data['purpose_id']}' ('internal_id')",
        )

    din_277, din_277_desc = tumonline_usage["din_277"].split(" - ")

    return data.setdefault(
        internal_id,
        {
            "id": internal_id,
            "type": "room",
            "name": f"{internal_id} ({room_alt_name})",
            "parents": data[b_id]["parents"] + [b_id],
            "props": {
                "ids": {
                    "roomcode": internal_id,
                    "arch_name": nat_data["number"],
                },
                "operator": {
                    "code": nat_data["org"]["org_code"],
                    "name": _(nat_data["org"]["org_name"], en_message=nat_data["org"]["org_name"]["en"]),
                    "url": f"https://campus.tum.de/tumonline/webnav.navigate_to?corg={nat_data['org']['org_id']}",
                    "id": nat_data["org"]["org_id"],
                },
            },
            "usage": {
                "name": _(nat_data["purpose"]["de"], en_message=nat_data["purpose"]["en"]),
                "din_277": din_277,
                "din_277_desc": din_277_desc,
            },
        },
    )


def _merge_nat_extra_props(r_data, nat_data):
    # pylint: disable=too-many-branches
    if nat_data.get("area", 0) > 0:
        r_data["props"].setdefault("stats", {}).setdefault("area_sqm", nat_data["area"])

    if nat_data.get("bauarbeiten"):
        r_data["props"].setdefault("construction_works", nat_data["bauarbeiten"])

    if nat_data.get("comment"):
        # German only, original NAT roomfinder property name "corona"
        r_data["props"].setdefault("covid_comment_html", nat_data["comment"])

    if nat_data.get("eexam"):
        eexams = nat_data["eexam"]["de"].rstrip(".").replace("Für", "für")
        r_data["props"].setdefault("eexams", _(eexams))

    if nat_data["schedule_url"] and "tumonline_data" not in r_data:
        r_data["props"].setdefault("calendar_url", nat_data["schedule_url"])

    if nat_data.get("seatings"):
        seatings = r_data["props"].setdefault("seatings", [])
        for seating in nat_data["seatings"]:
            seatings.append(
                {
                    "id": seating["seating_id"],
                    "name": _(seating["seating"], en_message=seating["seating_en"]),
                    "n_seats": seating["seat_count"],
                },
            )

    if nat_data.get("seats"):
        r_data["props"].setdefault("stats", {}).setdefault("n_seats", nat_data["seats"])

    if nat_data.get("steckdosen"):
        sockets = nat_data["steckdosen"]["de"].rstrip(".").replace("MItte", "Mitte")
        r_data["props"].setdefault("sockets", _(sockets))

    if nat_data.get("streaming"):
        html_parts = nat_data["streaming"]["de"].split(",")
        for link in html_parts:
            link = link.strip()
            if not link.startswith("<a") or not link.endswith("</a>"):
                raise RuntimeError(
                    f"Cannot process `streaming` property for '{r_data['id']}' (not a link): '{link}'",
                )

            href = link.split('href="')[1].split('" target=')[0]
            text = link.split('">')[1].split("</a>")[0]

            r_data["props"].setdefault("links", []).append(
                {
                    "text": _(text),
                    "url": href,
                },
            )

    if nat_data.get("teaching"):
        r_data["usage"].setdefault("teaching", nat_data["teaching"])
