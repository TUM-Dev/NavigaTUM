import json
import logging
import time
import urllib.request
from pathlib import Path
from urllib.error import HTTPError

CACHE_PATH = Path(__file__).parent / "cache"


def maybe_sleep(duration):
    """
    Sleep for the given duration, but only if the script was called during a workday and working hours.
    """
    if time.gmtime().tm_wday not in [5, 6] and 5 <= time.gmtime().tm_hour <= 22:
        time.sleep(duration)


def _write_cache_json(fname, data):
    with open(CACHE_PATH / fname, "w", encoding="utf-8") as file:
        json.dump(data, file, indent=4)


def _cached_json(fname):
    path = CACHE_PATH / fname
    if path.exists():
        with open(path, encoding="utf-8") as file:
            return json.load(file)
    return None


def _download_file(url, target_cache_file, quiet=False, quiet_errors=False):
    if not target_cache_file.exists():
        # url parameter does not allow path traversal, because we build it further up in the callstack
        try:
            urllib.request.urlretrieve(url, target_cache_file)  # nosec: B310
        except HTTPError as error:
            if not quiet_errors:
                logging.warning(f"GET {url} -> Failed to retrieve because: {error}")
            return None
        if not quiet:
            logging.warning(f"GET {url}")

    return target_cache_file
