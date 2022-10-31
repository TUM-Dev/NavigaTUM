import json
import time
from pathlib import Path

CACHE_PATH = Path(__file__).parent / "cache"


def maybe_sleep(duration):
    """
    Sleep for the given duration, but only if the script was called during a workday and working hours.
    """
    if time.gmtime().tm_wday not in [5, 6] and 5 <= time.gmtime().tm_hour <= 22:
        time.sleep(duration)


def _write_cache_json(fname, data):
    with open(CACHE_PATH / fname, "w", encoding="utf-8") as file:
        json.dump(data, file)


def _cached_json(fname):
    path = CACHE_PATH / fname
    if path.exists():
        with open(path, encoding="utf-8") as file:
            return json.load(file)
    return None
