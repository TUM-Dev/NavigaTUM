import json
import logging
from collections import Counter
from dataclasses import dataclass


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
        if _is_building_excluded(building.b_code):
            continue

        _merge_building(data, building)


def _infer_internal_id(b_code, data):
    # The NAT Roomfinder has buildings in it, that are not in TUMonline
    # (for example Max-Planck-Institut für Plasmaphysik). We keep them,
    # but use a different building id.
    if b_code.startswith("X"):
        if b_code == "XUCL":
            return "origins-cluster"
        else:
            return b_code[1:].lower()
    elif b_code in data:
        return b_code
    else:
        raise RuntimeError(
            f"Building id '{b_code}' not found in base data. "
            f"It may be missing in the areatree.",
        )


def _merge_building(data, building):
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


def _is_building_excluded(b_code):
    """A few buildings are not merged in for various reasons"""
    return b_code in {
        # 'Building' 0000 contains some buildings and places not in TUMonline as rooms
        "0000",
        # Teared down and replaced by 2330-2334 most likely
        "2301",
        "2302",
        "2303",
        "2304",
        "2305",
        "2306",
        "2307",
        "2308",
        # Unclear, but probably no longer existing or in use by TUM
        "2360",  # Pfortencontainer C1
        "2361",  # Umkleide-Werkstatt-Container C2
        "2904",  # Nymphenburgerstr. 39
        # "Institut für Grünlandlehre u. Haushaltstechnik"
        # Either no longer existing or it was renamed.
        "4201",
        # The following buildings should be in Dürnast from their ID, but according
        # to TUMonline they don't have any rooms or don't exist. It's hard to find
        # out where they are from online sources.
        "4504",  # STG.WST.-J. Viehhofstall
        "4507",  # STG.WST.-Trafo, Pumpstation
        "4511",  # STG.WST.-Biogasanlage
        "4525",  # STG.WST.-Mobiler Hühnerstall
        # Hirschau was sold to the Munich Airport by TUM:
        # https://www.merkur.de/lokales/erding/versuchsgut-hirschau-verkauft-247253.html
        "4701",
        "4702",
        "4703",
        "4704",
        "4705",
        "4706",
        "4707",
        "4708",
        "4709",
        "4710",
        "4711",
        "4712",
        "4713",
        "4714",
        "4715",
        "4716",
        "4717",
        "4718",
        "4719",
        "4720",
        "4721",
        # "Arb.Geb. {1,2,3}, SCHP" probably changed to 4914-4916
        "4911",
        "4912",
        "4913",
        # "Gästehaus" (5105) and "GRS Neubau" (5106). I can't find where they are.
        # the "Gästehaus" might be teared down already.
        "5105",
        "5106",
        # "RCM-Büro-Containerbau"; Has only rooms in the NAT roomfinder, but all of them
        # empty placeholders without data. Maybe no longer existing.
        "5269",
        # Former ITEM (Innovationszentrum für Therapeutische Medizintechnik), but placed
        # exactly where 5701 is now, probably no longer existing.
        # https://portal.mytum.de/pressestelle/pressemitteilungen/news-500)
        "5702",
        # Others
        # ¹WZW Source: https://wiki.tum.de/download/attachments/1318389295/141016_WZW_Plan_Final.pdf?api=v2
        "4118",  # "Kindervilla II"; Maybe ID change from 4118 to 4117
        "4198",  # "ehem. Wasserwerk"; Probably ID change from 4198 to 4192
        "4228",  # "Trafostation XI"; Maybe ID change from 4228 to 4281
        "4305",  # "Tierernährung NGB"; Probably now part of 4318¹
        "4306",  # "Tierernährung HGB"; Probably now part of 4318¹
        "4312",  # "Zierpflanzenbau Gewächshaus"; Probably ID change from 4311 to 4379¹
        "4313",  # "Zierpflanzenbau Wirtschaftsgebäude"; Can't find any location
        "4919",  # "Getreidesilo"; Maybe ID change from 4919 to 4010
        # Location unknown
        "2603",  # Tankstelle
        "4112",  # "Institutsgebäude Brauwesen"; Maybe 48.39508 / 11.72437 (near 4111)
        "4133",  # Wohngebäude Ganzenmüllerstr.
        "4280",  # Ehem. Arbeitsamt
        "4303",  # Vegetationsh. Inst. Pfl. und Züchtung
        "5216",  # CO 60 Quelle
        # They might be integrated custom somewhere else, but here we ignore these.
        "3002",  # "Testgebäude 2" => building which probably does not exist
        "5110",  # wurde Abgerissen
        "5537",  # "Interims-Zelt-TUMshop" => building which no longer exists
        "0598",
        "4298",
        "5538",
        "5998",  # "Interims-Tentomax => buildings no longer exist
        "5516",
        "5600",  # phantom buildings, which don't exist
        "XXXX",  # "virtueller Raum"
    }


def merge_nat_rooms(data):
    """
    Merge the rooms in the NAT Roomfinder with the existing data.
    This will not overwrite the existing data, but act directly on the provided data.
    """

    with open("external/results/rooms_nat.json", encoding="utf-8") as file:
        rooms = json.load(file)

    not_merged_parent = 0
    not_merged_outdated = 0
    for nat_id, nat_data in rooms.items():
        b_code, id_rest = nat_id.split(".", 1)
        if _is_building_excluded(b_code):
            not_merged_parent += 1
            continue

        b_id = _infer_internal_id(b_code, data)
        internal_id = b_id + "." + id_rest
        if _is_room_excluded(internal_id, b_id, data):
            not_merged_outdated += 1
            continue


    logging.debug(f"{not_merged_parent} rooms not merged because their parent buildings "
                  f"were not merged.")
    logging.debug(f"{not_merged_outdated} rooms not merged because their buildings "
                  f"are not exclusively from the NAT roomfinder (possibly outdated data).")


def _is_room_excluded(internal_id, b_id, data):
    if internal_id in data:
        return False
    else:
        building_sources = data[b_id].get("sources").get("base")
        # First source for buildings is always the areatree.
        if len(building_sources) == 2 and building_sources[1]["name"] == "NAT Roomfinder":
            return False
        else:
            return True

