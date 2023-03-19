import functools
import json
import logging
import time
import urllib.request
from pathlib import Path
from urllib.error import HTTPError

CACHE_PATH = Path(__file__).parent / "cache"


def clean_spaces(_string: str) -> str:
    """Remove leading and trailing spaces as well as duplicate spaces in-between"""
    return " ".join(_string.split())


def maybe_sleep(duration):
    """
    Sleep for the given duration, but only if the script was called during a workday and working hours.
    """
    if time.gmtime().tm_wday not in [5, 6] and 5 <= time.gmtime().tm_hour <= 22:
        time.sleep(duration)


def cached_json(filename: str):
    """
    Decorator which caches the functions' returned results in json format

    :filename: where to store the file
    """

    def decorator(func):  # needed, as we want to pass filename to the annotation
        decorator_filename = filename  # needed, as otherwise this context would be lost

        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            # prepare the filepath
            wrapper_filename = decorator_filename
            if args or kwargs:
                wrapper_filename = decorator_filename.format(*args, **kwargs)
            path = CACHE_PATH / wrapper_filename
            # get already existing file
            if path.exists():
                with open(path, encoding="utf-8") as file:
                    return json.load(file)
            # produce new file
            result = func(*args, **kwargs)
            with open(CACHE_PATH / wrapper_filename, "w", encoding="utf-8") as file:
                json.dump(result, file, indent=2, sort_keys=True)
            return result

        return wrapper

    return decorator


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
            logging.info(f"GET {url}")

    return target_cache_file
