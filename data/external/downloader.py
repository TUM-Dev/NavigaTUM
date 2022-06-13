# This script takes care of downloading data from the Roomfinder and TUMOnline
# and caching the results
import json
import os
import random
import string
import time
import urllib
import xmlrpc.client
import zipfile
from pathlib import Path

import requests
from bs4 import BeautifulSoup, element
from defusedxml import ElementTree as ET
from utils import convert_to_webp

ROOMFINDER_API_URL = "http://roomfinder.ze.tum.de:8192/"


def roomfinder_buildings():
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
        print(f"Retrieving {len(buildings)} buildings")
        for i, building in enumerate(buildings):
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
            time.sleep(0.05)
            if i % 10 == 0:
                print(".", end="", flush=True)
    print("")

    _write_cache_json(cache_name, buildings)
    return buildings


def roomfinder_rooms():
    """
    Retrieve the (extended, i.e. with coordinates) rooms data from the Roomfinder API.
    This may retrieve the Roomfinder buildings.

    :returns: A list of rooms, each room is a dict
    """
    cache_name = "rooms_roomfinder.json"

    rooms = _cached_json(cache_name)
    if rooms is not None:
        return rooms

    buildings = roomfinder_buildings()
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

    print(f"Retrieving {len(rooms_list)} rooms for {b_cnt} buildings")
    rooms = []
    for i, room in enumerate(rooms_list):
        extended_data = proxy.getRoomData(room)
        # for k, v in extended_data.items():
        #    rooms[i][k] = v
        extended_data["maps"] = proxy.getRoomMaps(room)
        extended_data["default_map"] = proxy.getDefaultMap(room)
        extended_data["metas"] = proxy.getRoomMetas(room)
        rooms.append(extended_data)
        time.sleep(0.05)
        if i % 10 == 0:
            print(".", end="", flush=True)
    print("")

    _write_cache_json(cache_name, rooms)
    return rooms


def _guess_queries(rooms, n_rooms):
    # First try: all single-digit numbers
    for i in range(10):
        if len(rooms) < n_rooms:
            time.sleep(0.05)
            yield str(i)
        else:
            return

    # Second try: all double-digit numbers
    for i in range(100):
        if len(rooms) < n_rooms:
            time.sleep(0.05)
            yield str(i).zfill(2)
        else:
            return

    # Thirs try: all characters
    for char in string.ascii_lowercase:
        if len(rooms) < n_rooms:
            time.sleep(0.05)
            yield char
        else:
            return


def roomfinder_maps():
    """
    Retrieve the maps including the data about them from Roomfinder.
    Map files will be stored in 'cache/maps/roomfinder'.

    This may retrieve Roomfinder rooms and buildings.

    :returns: A list of maps
    """
    cache_name = "maps_roomfinder.json"

    maps = _cached_json(cache_name)
    if maps is not None:
        return maps

    # The only way to get the map boundaries seems to be to download the kml with overlaid map.
    # For this api we need a room or building for each map available.
    rooms = roomfinder_rooms()
    buildings = roomfinder_buildings()

    used_maps = {}
    for building_entity in rooms + buildings:
        for _map in building_entity.get("maps", []):
            # _map[1] is the map id
            if _map[1] not in used_maps:
                if "r_id" in building_entity:
                    used_maps[_map[1]] = ("room", building_entity["r_id"], _map)
                else:
                    used_maps[_map[1]] = ("building", building_entity["b_id"], _map)

    maps = []
    for e_type, e_id, _map in used_maps.values():
        # Download as file
        url = f"http://roomfinder.ze.tum.de:8192/getMapImage?m_id={_map[1]}"
        filepath = f"maps/roomfinder/{_map[1]}.gif"
        _download_file(url, filepath)
        convert_to_webp(Path(filepath))

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

        rf_base_room_placemark_url = "https://portal.mytum.de/campus/roomfinder/getRoomPlacemark"
        if e_type == "room":
            url = f"{rf_base_room_placemark_url}?roomid={urllib.parse.quote_plus(e_id)}&mapid={_map[1]}"
            f_path = _download_file(url, f"maps/roomfinder/kmz/{_map[1]}.kmz")
        elif e_type == "building":
            url = f"{rf_base_room_placemark_url}?b_id={e_id}&mapid={_map[1]}"
            f_path = _download_file(url, f"maps/roomfinder/kmz/{_map[1]}.kmz")

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

    # Not all maps are used somewhere.
    # TODO: Download the rest

    _write_cache_json(cache_name, maps)
    return maps


def tumonline_areas():
    """
    Retrieve the building areas as in TUMOnline.

    :returns: A list of areas together with their id
    """
    filters = _get_roomsearch_xml(
        _get_tumonline_api_url("wbSuche.cbRaumForm"),
        {"pGebaeudebereich": 0},
        "filter/empty.xml",
    )

    return [{"id": int(e[0]), "name": e[1]} for e in _parse_filter_options(filters, "areas")]


def tumonline_usages_filter():
    """
    Retrieve the room usage types that are available as a filter in TUMOnline.
    These are not all usage types known to TUMOnline!

    :returns: A list of usage types together with their id
    """
    filters = _get_roomsearch_xml(
        _get_tumonline_api_url("wbSuche.cbRaumForm"),
        {"pVerwendung": 0},
        "filter/empty.xml",
    )

    return [{"id": int(e[0]), "name": e[1]} for e in _parse_filter_options(filters, "usages")]


def tumonline_buildings():
    """
    Retrieve the buildings as in TUMOnline with their assigned TUMOnline area.
    This may retrieve TUMOnline areas.

    :returns: A list of buildings, each building is a dict
    """
    cache_name = "buildings_tumonline.json"

    buildings = _cached_json(cache_name)
    if buildings is not None:
        return buildings

    filters = _get_roomsearch_xml(
        _get_tumonline_api_url("wbSuche.cbRaumForm"),
        {"pGebaeudebereich": 0},
        "filter/empty.xml",
    )
    all_buildings = _parse_filter_options(filters, "buildings")

    areas = tumonline_areas()
    buildings = []
    for area in areas:
        filters_area = _get_roomsearch_xml(
            _get_tumonline_api_url("wbSuche.cbRaumForm"),
            {"pGebaeudebereich": area["id"]},
            f"filter/area_{area['id']}.xml",
        )
        buildings_area = _parse_filter_options(filters_area, "buildings")
        for building in buildings_area:
            buildings.append({"filter_id": int(building[0]), "name": building[1], "area_id": area["id"]})

    # Not observed so far, I assume all buildings have an assigned area
    if len(buildings) != len(all_buildings):
        print("Warning: Not all buildings have an assigned area. Buildings without an area are discarded")

    _write_cache_json(cache_name, buildings)
    return buildings


def tumonline_rooms():
    """
    Retrieve the rooms as in TUMOnline including building and usage type.
    For some room types (e.g. lecture halls) additional information is retrieved.
    This may retrieve TUMOnline buildings.

    :returns: A list of rooms, each room is a dict
    """
    # To get both area/building and usage type for all buildings without needing to
    # query the details of all >30k rooms, the rooms are queried two times.
    # First for every area and building, second for every usage type.
    # This means that every room should be in a list exactly two times.
    # With 30k rooms this means over 1000 requests (max 30 rooms per page)

    # For these usages additional information is retrieved.
    extend_for_usages = {
        20,  # lecture hall
        41,  # seminar room
        55,  # Zeichensaal
        130,  # Unterrichtsraum
        131,  # Übungsraum
    }

    cache_name = "rooms_tumonline.json"

    rooms = _cached_json(cache_name)
    if rooms is not None:
        return rooms

    room_index = {}

    buildings = tumonline_buildings()
    for building in buildings:
        b_rooms = _retrieve_tumonline_roomlist("b", "building", "pGebaeude", building["filter_id"], building["area_id"])
        for room in b_rooms:
            room["b_filter_id"] = building["filter_id"]
            room["b_area_id"] = building["area_id"]
            room_index[room["roomcode"]] = room

    # Only a few usage types are named in the filter, however with their id it's also possible
    # to filter for other usage types. That's why we try them out.
    rooms = []
    usage_id = 1  # Observed: usage ids go up to 223, the limit below is for safety
    while not (usage_id > 300 or len(rooms) >= len(room_index)):
        u_rooms = _retrieve_tumonline_roomlist("u", "usage", "pVerwendung", usage_id)
        for room in u_rooms:
            room_index[room["roomcode"]]["usage"] = usage_id
            if usage_id in extend_for_usages:
                system_id = room_index[room["roomcode"]]["room_link"][24:]
                room_index[room["roomcode"]]["extended"] = _retrieve_tumonline_roominfo(system_id)
            rooms.append(room_index[room["roomcode"]])
        usage_id += 1

    _write_cache_json(cache_name, rooms)
    return rooms


def tumonline_usages():
    """
    Retrieve all usage types available in TUMOnline.
    This may retrieve TUMOnline rooms.

    :returns: A list of usages, each usage is a dict
    """
    cache_name = "usages_tumonline.json"

    usages = _cached_json(cache_name)
    if usages is not None:
        return usages

    rooms = tumonline_rooms()

    used_usage_types = {}
    for room in rooms:
        if room["usage"] not in used_usage_types:
            used_usage_types[room["usage"]] = room

    usages = []

    for usage_type, example_room in used_usage_types.items():
        # room links start with "wbRaum.editRaum?pRaumNr=..."
        system_id = example_room["room_link"][24:]
        roominfo = _retrieve_tumonline_roominfo(system_id)

        usage = roominfo["Basisdaten"]["Verwendung"]
        parts = []
        for prefix in ["(NF", "(VF", "(TF"]:
            if prefix in usage:
                parts = usage.split(prefix, 2)
                parts[1] = prefix + parts[1]
                break
        if len(parts) != 2:
            print(f"Unknown usage specification: {usage}")
            continue
        usage_name = parts[0].strip()
        usage_din_277 = parts[1].strip("()")

        usages.append({"id": usage_type, "name": usage_name, "din_277": usage_din_277})

    _write_cache_json(cache_name, usages)
    return usages


def tumonline_orgs():
    """
    Retrieve all organisations in TUMOnline, that may operate rooms.

    :returns: A dict of orgs like {org_code: {...}}
    """
    cache_name = "orgs-de_tumonline.json"

    orgs = _cached_json(cache_name)
    if orgs is not None:
        return orgs

    # There is also this URL, which is used to retrieve orgs that have courses,
    # but this is not merged in at the moment:
    # https://campus.tum.de/tumonline/ee/rest/brm.orm.search/organisations/chooser?$language=de&view=S_COURSE_LVEAB_ORG
    url = (
        "https://campus.tum.de/tumonline/ee/rest/brm.orm.search/organisations/chooser"
        "?$language=de"
        "&app=CO_LOC_GRUPPEN"
        "&view=CO_LOC_ORGCTX_PZ_ANONYM_V"
    )
    headers = {
        "Accept": "application/json",
    }

    # This is a single request, so not cached
    req = requests.get(url, headers=headers)
    if req.status_code != 200:
        raise RuntimeError(f"Failed to download organisations.\nrequest={req}\nrequest.text={req.text}")

    data = json.loads(req.text)

    try:
        results = data["resource"][0]["content"]["organisationChooserResultDto"]["searchResults"]
    except KeyError as error:
        raise RuntimeError(error) from error

    orgs = {}
    for _item in results:
        orgs[_item["designation"]] = {
            "id": _item["id"],
            "code": _item["designation"],
            "name": _item["name"],
            "path": _item["orgPath"],
        }

    _write_cache_json(cache_name, orgs)
    return orgs


def _retrieve_tumonline_roomlist(f_prefix, f_type, f_name, f_value, area_id=0):
    """Retrieve all rooms (multi-page) from the TUMOnline room search list"""
    cache_name = f"tumonline/{f_prefix}_{f_value}.{area_id}.json"

    all_rooms = _cached_json(cache_name)
    if all_rooms is not None:
        return all_rooms

    print(f"Retrieving {f_type} {f_value}")

    all_rooms = []
    pages_cnt = 1
    current_page = 0

    while current_page < pages_cnt:
        search_params = {
            "pStart": len(all_rooms) + 1,  # 1 + current_page * 30,
            "pSuchbegriff": "",
            "pGebaeudebereich": area_id,  # 0 for all areas
            "pGebaeude": 0,
            "pVerwendung": 0,
            "pVerwalter": 1,
            f_name: f_value,
        }
        req = requests.post("https://campus.tum.de/tumonline/wbSuche.raumSuche", data=search_params)
        rooms_on_page, pages_cnt, current_page = _parse_rooms_list(BeautifulSoup(req.text, "lxml"))
        all_rooms.extend(rooms_on_page)

        if current_page == 1:
            print(f"({pages_cnt}) ", end="")
        print(".", end="", flush=True)
        time.sleep(1.5)
    print("")

    _write_cache_json(cache_name, all_rooms)
    return all_rooms


def _retrieve_tumonline_roominfo(system_id):
    """Retrieve the extended room information from TUMOnline for one room"""
    html_parser: BeautifulSoup = _get_html(
        f"https://campus.tum.de/tumonline/wbRaum.editRaum?pRaumNr={system_id}",
        {},
        f"room/{system_id}",
    )

    roominfo = {}

    fieldsets = html_parser.find_all("fieldset", class_="MaskS")
    for fieldset in fieldsets:
        legend = fieldset.find("legend")
        table_name = legend.text.strip()
        if table_name in {"Basisdaten", "physikalische Eigenschaften", "Zusatzinformationen"}:
            roominfo[table_name] = {}

            table = fieldset.find("table")
            for row in table.find_all("tr"):
                columns = row.find_all("td")
                # Doesn't apply to the PLZ/Ort field, which has another table inside
                if len(columns) == 2:
                    roominfo[table_name][columns[0].text.strip()] = columns[1].text.strip()

    return roominfo


def _parse_filter_options(xml_parser: BeautifulSoup, filter_type):
    el_id = {
        "areas": "pGebaeudebereich",
        "buildings": "pGebaeude",
        "usages": "pVerwendung",
    }[filter_type]

    options = []

    sel = xml_parser.find("select", {"name": el_id})
    for opt in sel:
        if isinstance(opt, element.Tag) and opt.attrs["value"] != "0":
            options.append((opt.attrs["value"], opt.text))

    return options


def _parse_rooms_list(lxml_parser: BeautifulSoup):
    rooms = []

    table = lxml_parser.find("table", class_="list")

    if table is None:
        return [], 1, 1

    tbody = table.find("tbody")

    for row in tbody.find_all("tr"):
        columns = row.find_all("td")
        if len(columns) != 8:
            print(row)
            continue

        c_room = columns[1].find("a")
        c_calendar = columns[2].find("a")
        c_address = columns[5].find("a")
        c_operator = columns[7].find("a")
        data = {
            "list_index": columns[0].text,
            "roomcode": columns[1].text,
            "room_link": None if c_room is None else c_room.attrs["href"],
            "calendar": None if c_calendar is None else c_calendar.attrs["href"],
            "alt_name": columns[3].text,
            "arch_name": columns[4].text,
            "address": columns[5].text,
            "address_link": None if c_address is None else c_address.attrs["href"],
            "plz_place": columns[6].text,
            "operator": columns[7].text,
            "op_link": None if c_operator is None else c_operator.attrs["href"],
        }

        rooms.append(data)

    # Get information about number of pages
    pages_table = lxml_parser.find("table", class_="wr100")
    if pages_table is None:
        num_pages = 1
        current_page = 1
    else:
        columns = pages_table.find("tr").find_all("td")
        if len(columns) != 5:
            print(columns)
            raise RuntimeError("")

        num_pages = len(columns[3].find_all("option"))
        current_page = int(columns[3].find("option", selected=True).text)  # 1-indexed!

    return rooms, num_pages, current_page


def _get_cache_path(fname):
    return os.path.join(os.path.dirname(__file__), "cache", fname)


def _cached_json(fname):
    path = _get_cache_path(fname)
    if os.path.exists(path):
        with open(path, encoding="utf-8") as file:
            return json.load(file)
    else:
        return None


def _get_roomsearch_xml(url: str, params: dict, cache_fname: str) -> BeautifulSoup:
    root = _get_xml(url, params, cache_fname)
    elem = root.find('.//instruction[@jsid="raumSucheKontainerID"]')
    return BeautifulSoup(elem.text, "lxml")


def _get_xml(url: str, params: dict, cache_fname: str):
    cache_path = os.path.join(os.path.dirname(__file__), "cache", cache_fname)
    if os.path.exists(cache_path):
        tree = ET.parse(cache_path)
        return tree.getroot()

    print(f"GET {url}", params)
    req = requests.get(url, params)
    with open(cache_path, "w", encoding="utf-8") as file:
        file.write(req.text)
    return ET.fromstring(req.text)


def _get_html(url: str, params: dict, cache_fname: str) -> BeautifulSoup:
    cache_path = os.path.join(os.path.dirname(__file__), "cache", cache_fname)
    if os.path.exists(cache_path):
        with open(cache_path, encoding="utf-8") as file:
            return BeautifulSoup(file.read(), "lxml")
    else:
        req = requests.get(url, params)
        time.sleep(0.5)  # Not the best place to put this
        with open(cache_path, "w", encoding="utf-8") as file:
            file.write(req.text)
        return BeautifulSoup(req.text, "lxml")


def _download_file(url, cache_rel_path):
    cache_path = os.path.join(os.path.dirname(__file__), "cache", cache_rel_path)

    if not os.path.exists(cache_path):
        print(f"Retrieving: '{url}'")
        # url parameter does not allow path traversal, because we build it further up in the callstack
        urllib.request.urlretrieve(url, cache_path)  # nosec: B310

    return cache_path


def _write_cache_json(fname, data):
    path = _get_cache_path(fname)
    with open(path, "w", encoding="utf-8") as file:
        json.dump(data, file)


def _get_tumonline_api_url(base_target):
    # TODO: WTF is this ???
    magic_string = f"NC_{str(random.randint(0, 9999)).zfill(4)}"  # nosec: random is not used security/crypto purposes
    return f"https://campus.tum.de/tumonline/{base_target}/{magic_string}"
