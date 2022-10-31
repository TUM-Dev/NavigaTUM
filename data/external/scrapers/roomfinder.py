# This script takes care of downloading data from the Roomfinder and TUMonline
# and caching the results
import string
import urllib
import xmlrpc.client
import zipfile

from defusedxml import ElementTree as ET
from progress.bar import Bar  # type: ignore

from external.scraping_utils import maybe_sleep, _write_cache_json, _cached_json, CACHE_PATH
from utils import convert_to_webp

ROOMFINDER_API_URL = "http://roomfinder.ze.tum.de:8192"


def scrape_buildings():
    """
    Retrieve the (extended, i.e. with coordinates) buildings data from the Roomfinder API

    :returns: A list of buildings, each building is a dict
    """
    cache_name = "buildings_roomfinder.json"

    buildings = _cached_json(cache_name)
    if buildings is not None:
        return buildings

    with xmlrpc.client.ServerProxy(ROOMFINDER_API_URL) as proxy:
        buildings = proxy.getBuildings()
        bar = Bar("Retrieving", suffix="%(index)d / %(max)d buildings", max=len(buildings))
        for i, building in enumerate(buildings):
            bar.next()
            # Make sure b_id is numeric. There is an incorrect entry with the value
            # 'CiO/SGInstitute West, Bibliot' which causes a crash
            try:
                int(building["b_id"])
            except ValueError:
                continue
            extended_data = proxy.getBuildingData(building["b_id"])
            for key, value in extended_data.items():
                buildings[i][key] = value
            buildings[i]["maps"] = proxy.getBuildingMaps(building["b_id"])
            buildings[i]["default_map"] = proxy.getBuildingDefaultMap(building["b_id"])
            maybe_sleep(0.05)

    _write_cache_json(cache_name, buildings)
    return buildings


def scrape_rooms():
    """
    Retrieve the (extended, i.e. with coordinates) rooms data from the Roomfinder API.
    This may retrieve the Roomfinder buildings.

    :returns: A list of rooms, each room is a dict
    """
    cache_name = "rooms_roomfinder.json"

    rooms = _cached_json(cache_name)
    if rooms is not None:
        return rooms

    buildings = scrape_buildings()
    rooms_list = []

    # Get all rooms in a building
    # The API does not provide such a function directly, so we
    # have to use search for this. Since search returns a max
    # of 50 results we need to guess to collect all rooms.
    print("Searching for rooms in buildings")
    b_cnt = 0
    with xmlrpc.client.ServerProxy(ROOMFINDER_API_URL) as proxy:
        for building in buildings:
            if "b_roomCount" in building and building["b_roomCount"] > 0:
                search_results = proxy.searchRoom("", {"r_building": building["b_id"]})
                b_rooms = {room["r_id"] for room in search_results}

                if len(b_rooms) < building["b_roomCount"]:
                    # Collect guess queries that are executed until
                    # all buildings are found or the query list is exhausted
                    for guessed_query in _guess_queries(b_rooms, building["b_roomCount"]):
                        search_results = proxy.searchRoom(guessed_query, {"r_building": building["b_id"]})
                        b_rooms |= {r["r_id"] for r in search_results}

                    if len(b_rooms) < building["b_roomCount"]:
                        print("Could not guess all queries for:")

                b_cnt += 1
                print(f"{building['b_id']} -> {len(b_rooms)} / {building['b_roomCount']}")

                rooms_list.extend(list(b_rooms))

    rooms = []
    for room in Bar("Retrieving", suffix=f"%(index)d / %(max)d rooms for {b_cnt} buildings").iter(rooms_list):
        extended_data = proxy.getRoomData(room)
        # for k, v in extended_data.items():
        #    rooms[i][k] = v
        extended_data["maps"] = proxy.getRoomMaps(room)
        extended_data["default_map"] = proxy.getDefaultMap(room)
        extended_data["metas"] = proxy.getRoomMetas(room)
        rooms.append(extended_data)
        maybe_sleep(0.05)

    _write_cache_json(cache_name, rooms)
    return rooms


def _guess_queries(rooms, n_rooms):
    # First try: all single-digit numbers
    for i in range(10):
        if len(rooms) < n_rooms:
            maybe_sleep(0.05)
            yield str(i)
        else:
            return

    # Second try: all double-digit numbers
    for i in range(100):
        if len(rooms) < n_rooms:
            maybe_sleep(0.05)
            yield str(i).zfill(2)
        else:
            return

    # Thirs try: all characters
    for char in string.ascii_lowercase:
        if len(rooms) < n_rooms:
            maybe_sleep(0.05)
            yield char
        else:
            return


def scrape_maps():
    """
    Retrieve the maps including the data about them from Roomfinder.
    Map files will be stored in 'cache/maps/roomfinder'.

    This may retrieve Roomfinder rooms and buildings.

    :returns: A list of maps
    """
    cache_name = "maps_roomfinder.json"

    cached_maps = _cached_json(cache_name)
    if cached_maps is not None:
        return cached_maps

    # The only way to get the map boundaries seems to be to download the kml with overlaid map.
    # For this api we need a room or building for each map available.
    rooms = scrape_rooms()
    buildings = scrape_buildings()

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

    _write_cache_json(cache_name, maps)
    return maps


def _download_maps(used_maps):
    maps = []
    for e_type, e_id, _map in used_maps.values():
        # Download as file
        url = f"{ROOMFINDER_API_URL}/getMapImage?m_id={_map[1]}"
        filepath = CACHE_PATH / "maps" / "roomfinder" / f"rf{_map[1]}.gif"
        _download_file(url, filepath)
        convert_to_webp(filepath)

        map_data = {
            "id": _map[1],
            "scale": _map[0],
            "desc": _map[2],
            "width": _map[3],
            "height": _map[4],
        }
        maps.append(map_data)

        # Download as kmz to get the map boundary coordinates.
        # The world map (id 9) does not support kmz download
        if _map[1] == 9:
            continue

        f_path = _download_map(_map, e_id, e_type)

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


def _download_map(_map, e_id, e_type):
    filepath = CACHE_PATH / "maps" / "roomfinder" / "kmz" / f"{_map[1]}.kmz"
    if e_type == "room":
        base_url = "https://portal.mytum.de/campus/roomfinder/getRoomPlacemark"
        url = f"{base_url}?roomid={urllib.parse.quote_plus(e_id)}&mapid={_map[1]}"
        return _download_file(url, filepath)
    if e_type == "building":
        base_url = "https://portal.mytum.de/campus/roomfinder/getBuildingPlacemark"
        url = f"{base_url}?b_id={e_id}&mapid={_map[1]}"
        return _download_file(url, filepath)
    raise RuntimeError(f"Unknown entity type: {e_type}")


def _download_file(url, target_cache_file):
    if not target_cache_file.exists():
        print(f"Retrieving: '{url}'")
        # url parameter does not allow path traversal, because we build it further up in the callstack
        urllib.request.urlretrieve(url, target_cache_file)  # nosec: B310

    return target_cache_file
