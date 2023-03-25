# This script takes care of downloading data from the Roomfinder and TUMonline
# and caching the results
import json
import logging
import random

import requests
from bs4 import BeautifulSoup, element
from defusedxml import ElementTree as ET
from external.scraping_utils import CACHE_PATH, cached_json, maybe_sleep
from tqdm import tqdm

TUMONLINE_URL = "https://campus.tum.de/tumonline"


def scrape_areas():
    """
    Retrieve the building areas as in TUMonline.

    :returns: A list of areas together with their id
    """
    filters = _get_roomsearch_xml(
        _get_tumonline_api_url("wbSuche.cbRaumForm"),
        {"pGebaeudebereich": 0},
        "filter/empty.xml",
    )

    return [{"id": int(e[0]), "name": e[1]} for e in _parse_filter_options(filters, "areas")]


def scrape_usages_filter():
    """
    Retrieve the room usage types that are available as a filter in TUMonline.
    These are not all usage types known to TUMonline!

    :returns: A list of usage types together with their id
    """
    filters = _get_roomsearch_xml(
        _get_tumonline_api_url("wbSuche.cbRaumForm"),
        {"pVerwendung": 0},
        "filter/empty.xml",
    )

    return [{"id": int(e[0]), "name": e[1]} for e in _parse_filter_options(filters, "usages")]


@cached_json("buildings_tumonline.json")
def scrape_buildings():
    """
    Retrieve the buildings as in TUMonline with their assigned TUMonline area.
    This may retrieve TUMonline areas.

    :returns: A list of buildings, each building is a dict
    """

    areas = scrape_areas()
    logging.info("Scraping the buildings of tumonline")
    filters = _get_roomsearch_xml(
        _get_tumonline_api_url("wbSuche.cbRaumForm"),
        {"pGebaeudebereich": 0},
        "filter/empty.xml",
    )
    all_buildings = _parse_filter_options(filters, "buildings")

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
        logging.warning("Not all buildings have an assigned area. Buildings without an area are discarded")

    return sorted(buildings, key=lambda b: (b["name"], b["area_id"], b["filter_id"]))


@cached_json("rooms_tumonline.json")
def scrape_rooms():
    """
    Retrieve the rooms as in TUMonline including building and usage type.
    For some room types (e.g. lecture halls) additional information is retrieved.
    This may retrieve TUMonline buildings.

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

    buildings = scrape_buildings()

    logging.info("Scraping the rooms of tumonline")
    room_index = {}
    for building in buildings:
        b_rooms = _retrieve_roomlist(
            f_type="building",
            f_name="pGebaeude",
            f_value=building["filter_id"],
            area_id=building["area_id"],
        )
        for room in b_rooms:
            room["b_filter_id"] = building["filter_id"]
            room["b_area_id"] = building["area_id"]
            room_index[room["roomcode"]] = room

    # Only a few usage types are named in the filter, however with their id it's also possible
    # to filter for other usage types. That's why we try them out.
    rooms = []
    usage_id = 1  # Observed: usage ids go up to 223, the limit below is for safety
    while not (usage_id > 300 or len(rooms) >= len(room_index)):
        u_rooms = _retrieve_roomlist(f_type="usage", f_name="pVerwendung", f_value=usage_id, area_id=0)
        for room in u_rooms:
            room_index[room["roomcode"]]["usage"] = usage_id
            if usage_id in extend_for_usages:
                system_id = room_index[room["roomcode"]]["room_link"][24:]
                room_index[room["roomcode"]]["extended"] = _retrieve_roominfo(system_id)
            rooms.append(room_index[room["roomcode"]])
        usage_id += 1

    return sorted(rooms, key=lambda r: (r["list_index"], r["roomcode"]))


@cached_json("usages_tumonline.json")
def scrape_usages():
    """
    Retrieve all usage types available in TUMonline.
    This may retrieve TUMonline rooms.

    :returns: A list of usages, each usage is a dict
    """
    rooms = scrape_rooms()

    logging.info("Scraping the room-usages of tumonline")

    used_usage_types = {}
    for room in rooms:
        if room["usage"] not in used_usage_types:
            used_usage_types[room["usage"]] = room

    usages = []

    for usage_type, example_room in sorted(used_usage_types.items(), key=lambda u: u[0]):
        # room links start with "wbRaum.editRaum?pRaumNr=..."
        system_id = example_room["room_link"][24:]
        roominfo = _retrieve_roominfo(system_id)

        usage = roominfo["Basisdaten"]["Verwendung"]
        parts = []
        for prefix in ["(NF", "(VF", "(TF"]:
            if prefix in usage:
                parts = usage.split(prefix, 2)
                parts[1] = prefix + parts[1]
                break
        if len(parts) != 2:
            logging.warning(f"Unknown usage specification: {usage}")
            continue
        usage_name = parts[0].strip()
        usage_din_277 = parts[1].strip("()")

        usages.append({"id": usage_type, "name": usage_name, "din_277": usage_din_277})
    return usages


@cached_json("orgs-{lang}_tumonline.json")
def scrape_orgs(lang):
    """
    Retrieve all organisations in TUMonline, that may operate rooms.

    :params lang: 'en' or 'de'
    :returns: A dict of orgs like {org_code: {...}}
    """

    logging.info("Scraping the orgs of tumonline")
    # There is also this URL, which is used to retrieve orgs that have courses,
    # but this is not merged in at the moment:
    # https://campus.tum.de/tumonline/ee/rest/brm.orm.search/organisations/chooser?$language=de&view=S_COURSE_LVEAB_ORG
    url = f"{TUMONLINE_URL}/ee/rest/brm.orm.search/organisations?q=*&$language={lang}"
    headers = {"Accept": "application/json"}

    # This is a single request, so not cached
    req = requests.get(url, headers=headers, timeout=30)
    if req.status_code != 200:
        raise RuntimeError(f"Failed to download organisations.\nrequest={req}\nrequest.text={req.text}")

    data = json.loads(req.text)

    try:
        results = data["resource"]
    except KeyError as error:
        raise RuntimeError(error) from error

    orgs = {}
    for _item in results:
        item = _item["content"]["organisationSearchDto"]
        if "designation" in item:
            orgs[item["designation"]] = {
                "id": item["id"],
                "code": item["designation"],
                "name": item["name"],
                "path": item["orgPath"],
            }
    return orgs


@cached_json("tumonline/{f_value}.{area_id}.json")
def _retrieve_roomlist(f_type, f_name, f_value, area_id=0):
    """Retrieve all rooms (multi-page) from the TUMonline room search list"""

    all_rooms = []
    pages_cnt = 1
    current_page = 0

    with tqdm(desc=f"Searching Rooms for {f_type} {f_value}", total=pages_cnt, leave=False) as prog:
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
            req = requests.post(f"{TUMONLINE_URL}/wbSuche.raumSuche", data=search_params, timeout=30)
            rooms_on_page, pages_cnt, current_page = _parse_rooms_list(BeautifulSoup(req.text, "lxml"))
            all_rooms.extend(rooms_on_page)

            if prog.total != pages_cnt:
                prog.reset(pages_cnt)
            prog.update(1)
            maybe_sleep(1.5)
    return all_rooms


def _retrieve_roominfo(system_id):
    """Retrieve the extended room information from TUMonline for one room"""
    html_parser: BeautifulSoup = _get_html(
        f"{TUMONLINE_URL}/wbRaum.editRaum?pRaumNr={system_id}",
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
    el_id = {"areas": "pGebaeudebereich", "buildings": "pGebaeude", "usages": "pVerwendung"}[filter_type]

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
            logging.debug(row)
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
            logging.debug(columns)
            raise RuntimeError("")

        num_pages = len(columns[3].find_all("option"))
        current_page = int(columns[3].find("option", selected=True).text)  # 1-indexed!

    return rooms, num_pages, current_page


def _get_roomsearch_xml(url: str, params: dict, cache_fname: str) -> BeautifulSoup:
    root = _get_xml(url, params, cache_fname)
    elem = root.find('.//instruction[@jsid="raumSucheKontainerID"]')
    return BeautifulSoup(elem.text, "lxml")


def _get_xml(url: str, params: dict, cache_fname: str):
    cache_path = CACHE_PATH / cache_fname
    if cache_path.exists():
        tree = ET.parse(cache_path)
        return tree.getroot()

    logging.debug(f"GET {url}", params)
    req = requests.get(url, params, timeout=10)
    with open(cache_path, "w", encoding="utf-8") as file:
        file.write(req.text)
    return ET.fromstring(req.text)


def _get_html(url: str, params: dict, cache_fname: str) -> BeautifulSoup:
    cached_xml_file = CACHE_PATH / cache_fname
    if cached_xml_file.exists():
        with open(cached_xml_file, encoding="utf-8") as file:
            result = file.read()
    else:
        req = requests.get(url, params, timeout=10)
        maybe_sleep(0.5)  # Not the best place to put this
        with open(cached_xml_file, "w", encoding="utf-8") as file:
            result = req.text
            file.write(result)
    return BeautifulSoup(result, "lxml")


def _get_tumonline_api_url(base_target):
    # I have no idea, what this magic_string is, or why it exists..
    # Usage is the same as from TUMonline..
    magic_string = f"NC_{str(random.randint(0, 9999)).zfill(4)}"  # nosec: random is not used security/crypto purposes
    return f"{TUMONLINE_URL}/{base_target}/{magic_string}"
