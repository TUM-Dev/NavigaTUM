import itertools
import logging
import string
import urllib.parse
import xmlrpc.client
import zipfile
from pathlib import Path
from typing import Iterator, Literal, TypedDict

from defusedxml import ElementTree as ET
from external.scraping_utils import _download_file, CACHE_PATH, cached_json, maybe_sleep
from tqdm import tqdm
from utils import convert_to_webp

ROOMFINDER_API_URL = "http://roomfinder.ze.tum.de:8192"


@cached_json("buildings_roomfinder.json")
def scrape_buildings():
    """
    Retrieve the (extended, i.e. with coordinates) buildings data from the Roomfinder API

    :returns: A list of buildings, each building is a dict
    """
    logging.info("Scraping the buildings of the mytum roomfinder")

    with xmlrpc.client.ServerProxy(ROOMFINDER_API_URL) as proxy:
        buildings: list[dict] = proxy.getBuildings()
        for i, building in enumerate(tqdm(buildings, desc="Retrieving", unit="building")):
            # Make sure b_id is numeric. There is an incorrect entry with the value
            # 'CiO/SGInstitute West, Bibliot' which causes a crash
            try:
                int(building["b_id"])
            except ValueError:
                continue
            extended_data: dict[str, str] = proxy.getBuildingData(building["b_id"])
            for key, value in extended_data.items():
                buildings[i][key] = value
            buildings[i]["maps"] = proxy.getBuildingMaps(building["b_id"])
            for _map in buildings[i]["maps"]:
                _map[1] = f"rf{_map[1]}"
            buildings[i]["default_map"] = proxy.getBuildingDefaultMap(building["b_id"]) or None
            if default_map := buildings[i]["default_map"]:
                default_map[1] = f"rf{default_map[1]}"
            buildings[i]["b_room_count"] = buildings[i].pop("b_roomCount")
            maybe_sleep(0.05)

    return sorted(buildings, key=lambda m: m["b_id"])


class SearchResult(TypedDict):
    r_id: str


@cached_json("rooms_roomfinder.json")
def scrape_rooms():
    """
    Retrieve the (extended, i.e. with coordinates) rooms data from the Roomfinder API.
    This may retrieve the Roomfinder buildings.

    :returns: A list of rooms, each room is a dict
    """
    buildings = scrape_buildings()
    logging.info("Scraping the rooms of the mytum roomfinder")
    rooms_list = []

    # Get all rooms in a building
    # The API does not provide such a function directly, so we
    # have to use search for this. Since search returns a max
    # of 50 results we need to guess to collect all rooms.
    logging.info("Searching for rooms in each building")
    unreported_warnings = []
    with xmlrpc.client.ServerProxy(ROOMFINDER_API_URL) as proxy:
        for building in tqdm(buildings, desc="Guessing queries for building", unit="building"):
            if (b_room_count := building.get("b_room_count")) > 0:
                search_results: list[SearchResult] = proxy.searchRoom("", {"r_building": building["b_id"]})
                b_rooms = {room["r_id"] for room in search_results}

                if len(b_rooms) < b_room_count:
                    # Collect guess queries that are executed until
                    # all buildings are found or the query list is exhausted
                    for guessed_query in _guess_queries(b_rooms, b_room_count):
                        search_results = proxy.searchRoom(guessed_query, {"r_building": building["b_id"]})
                        b_rooms |= {r["r_id"] for r in search_results}

                if len(b_rooms) < b_room_count:
                    unreported_warnings.append(
                        f"Could not guess all queries for building {building['b_id']}, "
                        f"because |b_rooms|={len(b_rooms)} < b_room_count={building['b_room_count']}",
                    )
                rooms_list.extend(list(b_rooms))
    # reporting these issues here, to not fuck with tqdm
    for unreported_warning in unreported_warnings:
        logging.warning(unreported_warning)

    rooms = []
    for room in tqdm(rooms_list, desc=f"Retrieving {len(rooms_list)} rooms"):
        extended_data = proxy.getRoomData(room)
        # for k, v in extended_data.items():
        #    rooms[i][k] = v
        extended_data["metas"] = proxy.getRoomMetas(room)
        extended_data["maps"] = proxy.getRoomMaps(room)
        for _map in extended_data["maps"]:
            _map[1] = f"rf{_map[1]}"
        extended_data["default_map"] = proxy.getDefaultMap(room)
        if default_map := extended_data["default_map"][1]:
            default_map[1] = f"rf{default_map[1]}"
        rooms.append(extended_data)
        maybe_sleep(0.05)

    return sorted(rooms, key=lambda r: (r["b_id"], r["r_id"]))


def _guess_queries(rooms: list, n_rooms: int) -> Iterator[str]:
    """
    Iterates through all single/double character strings consisting of digit/ascii_lowercase to find successful queries

    Ordering because of number of entries:
    - single before double
    - digits before ascii_lowercase
    """
    for superset in [string.digits, string.ascii_lowercase]:
        for string_lenght in [1, 2]:
            for guess in itertools.product(superset, repeat=string_lenght):
                if len(rooms) >= n_rooms:
                    return
                maybe_sleep(0.05)
                yield "".join(guess)


@cached_json("maps_roomfinder.json")
def scrape_maps() -> list[dict]:
    """
    Retrieve the maps including the data about them from Roomfinder.
    Map files will be stored in 'cache/maps/roomfinder'.

    This may retrieve Roomfinder rooms and buildings.

    :returns: A list of maps
    """

    # The only way to get the map boundaries seems to be to download the kml with overlaid map.
    # For this api we need a room or building for each map available.
    rooms = scrape_rooms()
    buildings = scrape_buildings()

    logging.info("Scraping the rooms-maps of the mytum roomfinder")
    used_maps = {}
    for building_entity in rooms + buildings:
        for _map in building_entity.get("maps", []):
            # _map[1] is the map id
            if _map[1] not in used_maps:
                if "r_id" in building_entity:
                    used_maps[_map[1]] = ("room", building_entity["r_id"], _map)
                else:
                    used_maps[_map[1]] = ("building", building_entity["b_id"], _map)
    maps = _download_maps(used_maps)

    # Not all maps are used somewhere.
    # TODO: Download the rest

    return sorted(maps, key=lambda m: m["id"])


def _download_maps(used_maps):
    maps = []
    for e_type, e_id, _map in used_maps.values():
        # Download as file
        url = f"{ROOMFINDER_API_URL}/getMapImage?m_id={_map[1].removeprefix('rf')}"
        filepath = CACHE_PATH / "maps" / "roomfinder" / f"{_map[1]}.gif"
        _download_file(url, filepath)
        convert_to_webp(filepath)

        map_data = {
            "scale": _map[0],
            "id": _map[1],
            "desc": _map[2],
            "width": _map[3],
            "height": _map[4],
        }
        maps.append(map_data)

        # Download as kmz to get the map boundary coordinates.
        # The world map (id rf9) does not support kmz download
        if _map[1] == "rf9":
            continue

        f_path = _download_map(_map[1], e_id, e_type)

        with zipfile.ZipFile(f_path, "r") as zip_f, zip_f.open("RoomFinder.kml") as file:
            root = ET.fromstring(file.read())
            # <kml>[0] gives <Folder>,
            # <Folder>[3] gives <GroundOverlay>,
            # <GroundOverlay>[3] gives <LatLonBox>
            latlonbox = root[0][3][3]
            map_data["latlonbox"] = {
                "north": latlonbox[0].text,
                "east": latlonbox[1].text,
                "west": latlonbox[2].text,
                "south": latlonbox[3].text,
                "rotation": latlonbox[4].text,
            }
    return maps


def _download_map(_map_id: str, e_id: str, e_type: Literal["room", "building"]) -> Path | None:
    filepath = CACHE_PATH / "maps" / "roomfinder" / "kmz" / f"{_map_id}.kmz"
    if e_type == "room":
        base_url = "https://portal.mytum.de/campus/roomfinder/getRoomPlacemark"
        url = f"{base_url}?roomid={urllib.parse.quote_plus(e_id)}&mapid={_map_id.removeprefix('rf')}"
        return _download_file(url, filepath)
    if e_type == "building":
        base_url = "https://portal.mytum.de/campus/roomfinder/getBuildingPlacemark"
        url = f"{base_url}?b_id={e_id}&mapid={_map_id.removeprefix('rf')}"
        return _download_file(url, filepath)
    raise RuntimeError(f"Unknown entity type: {e_type}")
