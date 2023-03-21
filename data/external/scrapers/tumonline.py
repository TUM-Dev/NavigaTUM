# This script takes care of downloading data from the Roomfinder and TUMonline
# and caching the results
import json
import logging
import os
import random
from typing import Optional

import requests
from bs4 import BeautifulSoup, element
from defusedxml import ElementTree as ET
from external.scraping_utils import CACHE_PATH, cached_json, clean_spaces, maybe_sleep
from tqdm.contrib.concurrent import thread_map

TUMONLINE_URL = "https://campus.tum.de/tumonline"
XML_NAMESPACES = {"cor": "http://rdm.campusonline.at/"}


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


TUMONLINE_ROOM_XML_API_URL = f"{TUMONLINE_URL}j/ws/webservice_v1.0/rdm/rooms/xml"
TUMONLINE_XML_API_TOKEN = os.environ.get("TUMONLINE_API_TOKEN", "yeIKcuCGSzUCosnPZcKXkGeyUYGTQqUw")


@cached_json("rooms_tumonline.json")
def scrape_rooms():
    """
    Retrieve the rooms as in TUMonline including building and usage type.
    For some room types (e.g. lecture halls) additional information is retrieved.
    This may retrieve TUMonline buildings.

    :returns: A list of rooms, each room is a dict
    """
    logging.info("Generating tumonline-rooms")

    logging.info("Scraping the rooms of tumonline which have an operator")
    root = _get_xml(
        TUMONLINE_ROOM_XML_API_URL,
        {"token": TUMONLINE_XML_API_TOKEN, "orgUnitID": 1},
        "tumonline/rooms_with_ou.xml",
    )
    if root is None:
        raise RuntimeError("getting tumonline rooms with OU failed")

    logging.info("Repackaging the rooms with an operator into json")
    rooms = []
    for room in root.findall("cor:resource", XML_NAMESPACES):
        description = room.find("cor:description", XML_NAMESPACES)
        rooms.append(__parse_room(description))

    scraped_room_numbers = set(room["roomID"] for room in rooms)
    unscraped_room_numbers = set(range(0, max(scraped_room_numbers) + 5_000)) - scraped_room_numbers

    for room in thread_map(
        __scrape_unscraped_room_number,
        unscraped_room_numbers,
        desc="Scraping/repackaging the rooms without an operator",
        unit="unscraped_room_number",
    ):
        if room:
            rooms.append(room)

    # add extended datapoints
    def __apply_extended_roominfo(_room):
        maybe_sleep(random.randint(3, 6))  # nosec: not used for crypto, but because Threadpool maybe ineffective
        _room["extended"] = _retrieve_roominfo(system_id=_room["roomID"])

    thread_map(__apply_extended_roominfo, rooms, desc="Retrieving extra information", unit="room")

    # drop duplicate information
    for room in rooms:
        room["arch_name"] = room["extended"].pop("Zusatzinformationen", {}).get("Architekten-Raumnr.", None)
        for key in ["area", "purpose", "seats"]:
            if key in room:
                room.pop(key)
        relabelings = [
            ("additionalInformation", "alt_name"),
            ("purposeID", "usage"),
            ("roomID", "tumonline_room_nr"),
            ("roomCode", "roomcode"),
        ]
        for old, new in relabelings:
            room[new] = room.pop(old, None)

    return sorted(rooms, key=lambda r: r["roomcode"])


def __scrape_unscraped_room_number(room_number):
    maybe_sleep(random.randint(3, 6))  # nosec: not used for crypto, but because Threadpool maybe ineffective
    root = _get_xml(
        TUMONLINE_ROOM_XML_API_URL,
        {"token": TUMONLINE_XML_API_TOKEN, "roomID": room_number},
        f"tumonline/room_{room_number}.xml",
        quiet=True,
        quiet_errors=True,
    )
    if not root:
        return None
    logging.info(f"Found room at {room_number}")
    room = root.find("cor:resource", XML_NAMESPACES)
    description = room.find("cor:description", XML_NAMESPACES)
    return __parse_room(description)


def __parse_room(description: ET):
    room_data = {}
    # basic attributes
    for attr in description.findall("cor:attribute", XML_NAMESPACES):
        key = attr.attrib["{http://rdm.campusonline.at/}attrID"]
        link = attr.attrib.get("{http://rdm.campusonline.at/}attrAltUrl", None)
        if link:
            room_data[f"{key}_link"] = link
        # datatype correction
        if key in ["operator_id", "purposeID", "roomID", "seats"]:
            room_data[key] = int(attr.text)
        elif key == "area":
            room_data[key] = float(attr.text)
        else:
            room_data[key] = clean_spaces(attr.text)

    # orgs+external resources
    for resource_group in description.findall("cor:resourceGroup", XML_NAMESPACES):
        if resource_group.attrib["{http://rdm.campusonline.at/}typeID"] == "orgUnitUserList":
            room_data["operator_id"] = __parse_operator(
                resource_group.find(
                    "cor:description",
                    XML_NAMESPACES,
                ),
            )  # equipmentForExternalUseList is not added for lacking data quality
    return room_data


def __parse_operator(description: ET):
    operator_ids = []
    for org in description.findall("cor:resource", XML_NAMESPACES):
        org_data = {}
        org_description = org.find("cor:description", XML_NAMESPACES)
        for attr in org_description.findall("cor:attribute", XML_NAMESPACES):
            org_data[attr.attrib["{http://rdm.campusonline.at/}attrID"]] = attr.text

        # this hides a bunch of duplicate data
        operator_ids.append(int(org_data["orgUnitID"]))
    if len(operator_ids) > 1:
        raise RuntimeError("Ignoring second org for room")
    return operator_ids[0]


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
        roominfo = _retrieve_roominfo(system_id=example_room["tumonline_room_nr"])

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
            orgs[item["id"]] = {
                "id": item["id"],
                "code": item["designation"],
                "name": item["name"],
                "path": item["orgPath"],
            }
    return orgs


@cached_json("room/{system_id}.json")
def _retrieve_roominfo(system_id):
    """Retrieve the extended room information from TUMonline for one room"""
    req = requests.get(f"{TUMONLINE_URL}/wbRaum.editRaum?pRaumNr={system_id}", timeout=10)
    maybe_sleep(0.5)  # Not the best place to put this
    html_parser = BeautifulSoup(req.text, "lxml")

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
                    roominfo[table_name][clean_spaces(columns[0].text)] = clean_spaces(columns[1].text)

    return roominfo


def _parse_filter_options(xml_parser: BeautifulSoup, filter_type):
    el_id = {"areas": "pGebaeudebereich", "buildings": "pGebaeude", "usages": "pVerwendung"}[filter_type]

    options = []

    sel = xml_parser.find("select", {"name": el_id})
    for opt in sel:
        if isinstance(opt, element.Tag) and opt.attrs["value"] != "0":
            options.append((opt.attrs["value"], opt.text))

    return options


def _get_roomsearch_xml(url: str, params: dict, cache_fname: str) -> BeautifulSoup:
    for _ in range(5):
        root = _get_xml(url, params, cache_fname)
        if root is not None:
            elem = root.find('.//instruction[@jsid="raumSucheKontainerID"]')
            return BeautifulSoup(elem.text, "lxml")
    raise RuntimeError(f"getting {url} failed 5x => cannot complete operation")


def _get_xml(url: str, params: dict, cache_fname: str, quiet=False, quiet_errors=False) -> Optional[ET.XML]:
    cache_path = CACHE_PATH / cache_fname
    if cache_path.exists():
        tree = ET.parse(cache_path)
        return tree.getroot()

    if not quiet:
        logging.debug(f"GET {url}", params)
    req = requests.get(url, params, timeout=10)
    if req.status_code != 200:
        if not quiet_errors:
            logging.debug(f"|->\t{url} is unscrape-able", params)
        return None

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
