import itertools
import json
import logging
import string
import urllib.parse
import xmlrpc.client  # nosec: B411
import zipfile
from collections.abc import Iterator
from pathlib import Path
from typing import Literal, TypedDict

import requests
import utm
from defusedxml import ElementTree as ET
from tqdm import tqdm

from external.scraping_utils import _download_file, CACHE_PATH, maybe_sleep
from utils import convert_to_webp

ROOMFINDER_API_URL = "http://roomfinder.ze.tum.de:8192"


def _sanitise_building(building: dict):
    for _map in building["maps"]:
        _map[1] = f"rf{_map[1]}"
    if default_map := building["default_map"]:
        default_map[1] = f"rf{default_map[1]}"
    building["b_room_count"] = building.pop("b_roomCount")
    # the Building "Sonstige" does not have a valid lat/lon => we chose the main campus of TUM as a default
    zone_number = int(building.pop("utm_zone"))
    easting = building.pop("utm_easting")
    northing = building.pop("utm_northing")
    if building["b_id"] == "0000":
        building["lat"], building["lon"] = 48.14903, 11.56735
    else:
        building["lat"], building["lon"] = _utm_to_latlon(easting, northing, zone_number)


def scrape_buildings() -> None:
    """Retrieve the (extended, i.e. with coordinates) buildings data from the Roomfinder API"""
    logging.info("Scraping the buildings of the mytum roomfinder")

    with xmlrpc.client.ServerProxy(ROOMFINDER_API_URL) as proxy:
        buildings: list[dict] = proxy.getBuildings()
        for building in tqdm(buildings, desc="Retrieving", unit="building"):
            # Make sure b_id is numeric. There is an incorrect entry with the value
            # 'CiO/SGInstitute West, Bibliot' which causes a crash
            try:
                int(building["b_id"])
            except ValueError:
                continue
            extended_data = proxy.getBuildingData(building["b_id"])
            building.update(**extended_data)
            building["maps"] = proxy.getBuildingMaps(building["b_id"])
            building["default_map"] = proxy.getBuildingDefaultMap(building["b_id"]) or None
            _sanitise_building(building)
            maybe_sleep(0.01)

    buildings = sorted(buildings, key=lambda m: m["b_id"])
    with open(CACHE_PATH / "buildings_roomfinder.json", "w", encoding="utf-8") as file:
        json.dump(buildings, file, indent=2, sort_keys=True)


def _utm_to_latlon(easting: float, northing: float, zone_number: int, zone_letter: str = "U") -> tuple[float, float]:
    # UTM zone is either 32 or 33, corresponding to zones "32U" and "33U"
    # TODO: Map image boundaries also included "33T". It could maybe be possible to guess
    #       whether it is "U" or "T" based on the northing (which is always the distance
    #       to the equator).
    utm.check_valid_zone(zone_number, zone_letter)
    if zone_number not in {32, 33}:
        raise RuntimeError(f"Unexpected UTM zone '{zone_number}'")
    return utm.to_latlon(easting, northing, zone_number, zone_letter)


class SearchResult(TypedDict):
    r_id: str


def _sanitise_room(room: dict) -> dict:
    for _map in room["maps"]:
        _map[1] = f"rf{_map[1]}"
    if default_map := room["default_map"]:
        default_map[1] = f"rf{default_map[1]}"
    room["lat"], room["lon"] = _utm_to_latlon(
        zone_number=int(room.pop("utm_zone")),
        easting=room.pop("utm_easting"),
        northing=room.pop("utm_northing"),
    )
    return room


def scrape_rooms() -> None:
    """
    Retrieve the (extended, i.e. with coordinates) rooms data from the Roomfinder API.

    This may retrieve the Roomfinder buildings.
    """
    with xmlrpc.client.ServerProxy(ROOMFINDER_API_URL) as proxy:
        room_ids = _get_all_rooms_for_all_buildings(proxy)
        logging.info("Scraping the rooms of the mytum roomfinder")
        rooms = []
        for room_id in tqdm(room_ids, desc=f"Retrieving {len(room_ids)} rooms"):
            room = proxy.getRoomData(room_id)
            room["metas"] = proxy.getRoomMetas(room_id)
            room["maps"] = proxy.getRoomMaps(room_id)
            room["default_map"] = proxy.getDefaultMap(room_id)
            rooms.append(_sanitise_room(room))
            maybe_sleep(0.01)

    rooms = sorted(rooms, key=lambda r: (r["b_id"], r["r_id"]))
    with open(CACHE_PATH / "rooms_roomfinder.json", "w", encoding="utf-8") as file:
        json.dump(rooms, file, indent=2, sort_keys=True)


def _get_all_rooms_for_all_buildings(proxy: xmlrpc.client.ServerProxy) -> list:
    """
    Get all rooms in a building

    The API does not provide such a function directly, so we have to use search for this.
    Since search returns a max of 50 results we need to guess to collect all rooms.
    """
    logging.info("Searching for rooms in each building")
    with open(CACHE_PATH / "buildings_roomfinder.json", encoding="utf-8") as file:
        buildings = json.load(file)
    unreported_warnings = []
    rooms_list = []
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
                    f"because {len(b_rooms)=} < {building['b_room_count']=}",
                )
            rooms_list.extend(list(b_rooms))
    # reporting these issues here, to not fuck with tqdm
    for unreported_warning in unreported_warnings:
        logging.warning(unreported_warning)
    return rooms_list


def _guess_queries(rooms: set[str], n_rooms: int) -> Iterator[str]:
    """
    Iterate through all single/double character strings consisting of digit/ascii_lowercase to find successful queries

    Ordering because of number of entries:
    - single before double
    - digits before ascii_lowercase
    """
    for superset in [string.digits, string.ascii_lowercase]:
        for string_lenght in [1, 2]:
            for guess in itertools.product(superset, repeat=string_lenght):
                if len(rooms) >= n_rooms:
                    return
                maybe_sleep(0.01)
                yield "".join(guess)


def scrape_maps() -> None:
    """
    Retrieve the maps including the data about them from Roomfinder.

    Map files will be stored in 'cache/maps/roomfinder'.
    """
    # The only way to get the map boundaries seems to be to download the kml with overlaid map.
    # For this api we need a room or building for each map available.
    with open(CACHE_PATH / "rooms_roomfinder.json", encoding="utf-8") as file:
        rooms = json.load(file)
    with open(CACHE_PATH / "buildings_roomfinder.json", encoding="utf-8") as file:
        buildings = json.load(file)

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

    maps = sorted(maps, key=lambda m: m["id"])
    with open(CACHE_PATH / "maps_roomfinder.json", "w", encoding="utf-8") as file:
        json.dump(maps, file, indent=2, sort_keys=True)


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
        f_path.unlink()
    return maps


def _download_map(_map_id: str, e_id: str, e_type: Literal["room", "building"]) -> Path | None:
    filepath = CACHE_PATH / "maps" / "roomfinder" / "kmz" / f"{_map_id}.kmz"
    if e_type == "room":
        base_url = "https://portal.mytum.de/campus/roomfinder/getRoomPlacemark"
        url = f"{base_url}?roomid={urllib.parse.quote_plus(e_id)}&mapid={_map_id.removeprefix('rf')}"
        try:
            return _download_file(url, filepath)
        except requests.exceptions.RequestException:
            return None
    if e_type == "building":
        base_url = "https://portal.mytum.de/campus/roomfinder/getBuildingPlacemark"
        url = f"{base_url}?b_id={e_id}&mapid={_map_id.removeprefix('rf')}"
        try:
            return _download_file(url, filepath)
        except requests.exceptions.RequestException:
            return None
    raise RuntimeError(f"Unknown entity type: {e_type}")
