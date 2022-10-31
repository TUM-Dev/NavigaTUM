# This script takes care of downloading data from the school of natural sciences (oiginally the physics department)
import json
import logging
from multiprocessing.pool import ThreadPool

from tqdm import tqdm
from tqdm.contrib.concurrent import thread_map

from external.scraping_utils import _cached_json, _download_file, _write_cache_json, CACHE_PATH

NAT_API_URL = "https://api.srv.nat.tum.de/api/v1/rom"
NAT_CACHE_DIR = CACHE_PATH / "nat"


def scrape_buildings():
    """
    Retrieve the buildings as in the NAT roomfinder.
    """
    cache_name = "buildings_nat.json"

    buildings = _cached_json(cache_name)
    if buildings is not None:
        return buildings

    logging.info("Scraping the buildings of the NAT")
    _download_file(f"{NAT_API_URL}/building", CACHE_PATH / cache_name)

    return _cached_json(cache_name)


def scrape_rooms():
    """
    Retrieve the rooms as in the NAT roomfinder.

    :returns: A list of rooms, each room is a dict
    """
    cache_name = "rooms_nat.json"

    rooms = _cached_json(cache_name)
    if rooms is not None:
        return rooms

    logging.info("Scraping the rooms of the NAT")
    base_info = _get_base_room_infos()
    rooms = {}
    for room in thread_map(_download_and_merge_room, base_info, desc="Downloaded nat rooms"):
        key = room["room_code"]  # needed, as room_code is removed in _sanitise_room
        rooms[key] = _sanitise_room(room)

    _write_cache_json(cache_name, rooms)
    return rooms


def _sanitise_room(room: dict):
    """
    Sanitise the room information
    """
    room.pop("room_code")

    return room


def _merge(content, base):
    """
    Merge the base information into the room content
    """
    for key, value in base.items():
        if key not in content or content[key] is None:
            content[key] = value
        elif content[key] != value:
            raise RuntimeError(f"Warning: {key} differs for {base['room_code']}: {content[key]} vs {value}")
    return content


def _download_and_merge_room(base):
    """
    Download the room information and merge it with the base information.
    """
    room_code = base["room_code"]
    target_filepath = NAT_CACHE_DIR / f"room_{room_code}.json"
    downloaded_file = _download_file(f"{NAT_API_URL}/{room_code}", target_filepath, quiet=True)
    if not downloaded_file:
        return base
    content = json.loads(downloaded_file.read_text(encoding="utf-8"))
    return _merge(content, base)


def _get_base_room_infos():
    """
    The API is a bit buggy and some rooms are throwing 500 errors
    => we need to do with binary search workaround
    """
    cache_name = NAT_CACHE_DIR / "base_info.json"

    total_hits = _cached_json(cache_name)
    if total_hits is not None:
        return total_hits

    # download the provided ids in chunks (the API is only offering chunks of 5_000)
    undownloadable = []
    work_queue = list((i, 5_000) for i in range(0, 50_000, 5_000))
    pool = ThreadPool()
    with tqdm(desc="Downloaded nat base room info", total=50_000) as prog:
        while work_queue:
            new_queue = []  # modifiying work_queue while iterating over it is a bad idea
            for (start, batch), dl in pool.starmap(_try_download_room_base_info, work_queue):
                # there may be files which we did not download due to one error...

                if dl or batch == 1:
                    prog.update()

                if not dl and batch != 1:
                    new_batch = batch // 2
                    new_queue.append((start, new_batch))
                    new_queue.append((start + new_batch, batch - new_batch))
                if not dl and batch == 1:
                    undownloadable.append(start)
            work_queue = new_queue

    total_hits = _join_room_hits()
    _write_cache_json(cache_name, total_hits)
    if undownloadable:  # down here to make shure, that tdtm has flushed the output
        _report_undownloadable(undownloadable)
    return total_hits


def _try_download_room_base_info(start: int, batch: int):
    dl = _download_file(
        f"{NAT_API_URL}/?limit={batch}&offset={start}",
        NAT_CACHE_DIR / f"rooms_base_{start}_to_{(start + 1) * batch - 1}.json",
        quiet=True,
        quiet_errors=True,
    )
    return (start, batch), dl


def _report_undownloadable(undownloadable: list[int]):
    """
    Report the undownloadable rooms in a range based format
    """
    undownloadable.sort()

    logging.warning("The following spans could not be downloaded:")
    # find the ranges in the sorted array
    start = undownloadable[0]
    end = undownloadable[0]
    for i in range(1, len(undownloadable)):
        if undownloadable[i] == end + 1:
            end = undownloadable[i]
        else:
            if start == end:
                logging.warning(f"\t{start}")
            else:
                logging.warning(f"\t{start}->{end} ({end - start + 1} rooms)")
            start = undownloadable[i]
            end = undownloadable[i]


def _join_room_hits():
    """
    Join the hits (which are chunked until this point) into one single index to run requests for
    """
    total_hits = []
    for file_path in NAT_CACHE_DIR.iterdir():
        if not file_path.name.startswith("rooms_base_"):
            continue
        with open(file_path, encoding="utf-8") as f:
            total_hits.extend(json.load(f)["hits"])
    return total_hits
