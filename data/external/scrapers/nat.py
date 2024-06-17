# This script takes care of downloading data from the school of natural sciences (oiginally the physics department)
import json
import logging
from multiprocessing.pool import ThreadPool
from pathlib import Path

import backoff
import requests
from tqdm import tqdm
from tqdm.contrib.concurrent import thread_map

from external.scraping_utils import _download_file, CACHE_PATH
from utils import TranslatableStr as _, setup_logging

NAT_API_URL = "https://api.srv.nat.tum.de/api/v1/rom"
NAT_CACHE_DIR = CACHE_PATH / "nat"


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def scrape_buildings():
    """Retrieve the buildings as in the NAT roomfinder."""
    logging.info("Scraping the buildings of the NAT")
    buildings = requests.get(f"{NAT_API_URL}/building", timeout=30).json()

    # TODO: Remove if NAT does fix this.. TUM changed names in denazification initative
    for building in buildings:
        building["building_name"].replace("Bestelmeyer Süd", "Zentralgebäude 2")
        building["building_name"].replace("Bestelmeyer Nord", "Zentralgebäude 7")

    with open(CACHE_PATH / "buildings_nat.json", "w", encoding="utf-8") as file:
        json.dump(buildings, file, indent=2, sort_keys=True)


def scrape_rooms() -> None:
    """Retrieve the rooms as in the NAT roomfinder."""
    logging.info("Scraping the rooms of the NAT")
    base_info = _get_base_room_infos()
    rooms = {}
    for room in thread_map(_download_and_merge_room, base_info, desc="Downloaded nat rooms"):
        if not room:
            continue  # we skip unprocessable rooms
        key = room["room_code"]  # needed, as room_code is removed in _sanitise_room
        rooms[key] = _sanitise_room(room)

    _extract_orgs(rooms)
    _extract_campus(rooms)

    with open(CACHE_PATH / "rooms_nat.json", "w", encoding="utf-8") as file:
        json.dump(rooms, file, indent=2, sort_keys=True)


def _extract_orgs(rooms: dict) -> None:
    """Extract the organisations from the room information."""
    logging.info("Extracting orgs from the rooms")
    orgs = {}
    for room in rooms.values():
        org = room.pop("org")
        if not org["deleted"]:
            # remove not-usefull fields
            org.pop("deleted")
            org.pop("org_nameshort")  # Always `null`

            org_id = org.pop("org_id")
            if isinstance(org["org_name"], str):
                org["org_name"] = _(org["org_name"])
            room["org_id"] = org_id
            orgs[org_id] = org

    with open(CACHE_PATH / "orgs_nat.json", "w", encoding="utf-8") as file:
        json.dump(orgs, file, indent=2, sort_keys=True)


def _extract_campus(rooms: dict) -> None:
    """Extract the organisations from the room information."""
    logging.info("Extracting orgs from the rooms")
    campi = {}
    for room in rooms.values():
        if campus := room.pop("campus"):
            campus_id = str(campus.pop("campus_id"))
            if isinstance(campus["campus"], str):
                campus["campus"] = _(campus["campus"], en_message=campus["campus"])
            if isinstance(campus["campusshort"], str):
                campus["campusshort"] = _(campus["campusshort"], en_message=campus["campusshort"])
            campi[campus_id] = campus
        else:
            campus_id = None
        room["campus_id"] = campus_id

    with open(CACHE_PATH / "campus_nat.json", "w", encoding="utf-8") as file:
        json.dump(campi, file, indent=2, sort_keys=True)


def _extract_translations(item: dict) -> None:
    """
    De-Inline the translations of keys into dicts.

    Example: {"key": "A", "key_en": "B"} will be transformed into {"key": {"de": "A", "en": "B"}}
    """
    translatable_keys: list[tuple[str, str]] = [(k.removesuffix("_en"), k) for k in item if k.endswith("_en")]
    for key, key_en in translatable_keys:
        eng = item.pop(key_en)
        if eng and item[key] and eng != item[key]:
            item[key] = {"de": item[key] or eng, "en": eng}


def _extract_translation_recursive(item):
    """De-Inline the translations of keys into dicts, recursively for all dicts in dicts or lists."""
    if isinstance(item, dict):
        for sub_item in item.values():
            _extract_translation_recursive(sub_item)
        _extract_translations(item)
    if isinstance(item, list):
        for sub_item in item:
            _extract_translation_recursive(sub_item)


def _sanitise_room(room: dict) -> dict:
    """
    Sanitise the room information.

    After this step:
    - fields are converted to our naming and partially our layout
    - all fields, which are supposed to be translatable are converted to our format and maybe manually translated
    """
    # fixing translations
    room["purpose"] = _(room["purpose"]["purpose"])

    _extract_translation_recursive(room)
    if campus_id := room["building"].pop("campus_id"):
        room["building"]["campus"]["campus_id"] = campus_id
    if isinstance(room["steckdosen"], str):
        room["steckdosen"] = _(room["steckdosen"].rstrip("."))
    room["campus"] = room["building"].pop("campus")

    # bauarbeiten is a str, indicating if something is wrong on in the room
    # accuracy is doubtfull sometimes => removal to prevent issues down the line
    room.pop("bauarbeiten")

    # badly maintained, to some part outdated
    room.pop("corona")
    room.pop("corona_ready")
    if room["eexam"]:
        room["eexam"] = _(room["eexam"].rstrip(".").replace("Für", "für"))

    # coordinates: there are two sets of coordinates on each entry.
    # This function makes sure, that they are the same
    _extract_coords(room)

    room["id"] = room.pop("room_code")  # our naming is id for this datapoint

    room.pop("orgs")  # nat internal org, not useful for us or consistent enough
    room.pop("building")  # scraped in a previous step

    fields_with_no_information = [
        "override_seats",
        "override_teaching",
        "modified",
        "contact",
        "events_end",
        "events_start",
    ]
    for field_name in fields_with_no_information:
        room.pop(field_name)
    return room


def _extract_coords(room):
    lat = room.pop("latitude")
    room_lat = room.pop("room_latitude")
    if lat and room_lat and lat != room_lat:
        logging.warning(f"Room {room['room_code']} has different latitudes: {lat} vs {room_lat}")
    lon = room.pop("longitude")
    room_lon = room.pop("room_longitude")
    if lon and room_lon and lon != room_lon:
        logging.warning(f"Room {room['room_code']} has different longitudes: {lon} vs {room_lon}")

    room["coordinates"] = {"lat": lon or room_lat, "lon": lon or room_lon, "source": "NAT"}


def _merge(content, base):
    """Merge the base information into the room content"""
    for key, value in base.items():
        if key not in content or content[key] is None:
            content[key] = value
        elif content[key] != value:
            raise RuntimeError(f"Warning: {key} differs for {base['room_code']}: {content[key]} vs {value}")
    return content


def _download_and_merge_room(base):
    """Download the room information and merge it with the base information."""
    room_code = base["room_code"]
    target_filepath = NAT_CACHE_DIR / f"room_{room_code}.json"
    try:
        downloaded_file = _download_file(f"{NAT_API_URL}/{room_code}", target_filepath)
    except requests.exceptions.RequestException:
        return None
    content = json.loads(downloaded_file.read_text(encoding="utf-8"))
    for useless_key in ["events_end", "events_start"]:
        if useless_key in content:
            del content[useless_key]
    content["area"] = float(content["area"])
    base["area"] = float(base["area"])
    return _merge(content, base)


def _get_base_room_infos():
    """
    Download the base room information

    The API is a bit buggy and some rooms are throwing 500 errors
    => we need to do with binary search workaround
    """
    # download the provided ids in chunks (the API is only offering chunks of 5_000)
    undownloadable: list[int] = []
    work_queue = [(i, 5_000) for i in range(0, 50_000, 5_000)]
    pool = ThreadPool()
    with tqdm(desc="Downloaded nat base room info", total=50_000) as prog:
        while work_queue:
            new_queue: list[tuple[int, int]] = []  # modifiying work_queue while iterating over it is a bad idea
            for (start, batch), downloaded_file in pool.starmap(_try_download_room_base_info, work_queue):
                # there may be files which we did not download due to one error...

                if downloaded_file or batch == 1:
                    prog.update(batch)

                if not downloaded_file:
                    if batch == 1:
                        undownloadable.append(start)
                    else:
                        new_batch = batch // 2
                        new_queue.extend(((start, new_batch), (start + new_batch, batch - new_batch)))
            work_queue = new_queue

    total_hits = _join_room_hits()
    if undownloadable:  # down here to make sure, that tqdm has flushed the output
        _report_undownloadable(undownloadable)
    return total_hits


def _try_download_room_base_info(start: int, batch: int) -> tuple[tuple[int, int], Path | None]:
    try:
        url = f"{NAT_API_URL}/?limit={batch}&offset={start}"
        file_path = NAT_CACHE_DIR / f"rooms_base_{start}_to_{start + batch - 1}.json"
        return (start, batch), _download_file(url, file_path)
    except requests.exceptions.RequestException:
        return (start, batch), None


def _report_undownloadable(undownloadable: list[int]) -> None:
    """Report the undownloadable rooms in a range based format"""
    undownloadable.sort()

    logging.warning("The following spans could not be downloaded:")
    # find the ranges in the sorted array
    start = undownloadable[0]
    end = undownloadable[0]
    for i in range(1, len(undownloadable)):
        if undownloadable[i] != end + 1:
            if start == end:
                logging.warning(f"\t{start}")
            else:
                logging.warning(f"\t{start}->{end} ({end - start + 1} rooms)")
            start = undownloadable[i]
        end = undownloadable[i]


def _join_room_hits():
    """Join the hits (which are chunked until this point) into one single index to run requests for"""
    total_hits = []
    for file_path in NAT_CACHE_DIR.iterdir():
        if not file_path.name.startswith("rooms_base_"):
            continue
        with open(file_path, encoding="utf-8") as file:
            total_hits.extend(json.load(file)["hits"])
    return total_hits


if __name__ == "__main__":
    setup_logging(level=logging.INFO)
    NAT_CACHE_DIR.mkdir(exist_ok=True)
    scrape_buildings()
    scrape_rooms()
