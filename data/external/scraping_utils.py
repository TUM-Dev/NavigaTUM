import functools
import json
import logging
import time
import urllib.request
from pathlib import Path
from typing import Callable, ParamSpec, TypeVar
from urllib.error import HTTPError

CACHE_PATH = Path(__file__).parent / "cache"


def maybe_sleep(duration: float) -> None:
    """
    Sleep for the given duration, but only if the script was called during a workday and working hours.
    """
    if time.gmtime().tm_wday not in [5, 6] and 5 <= time.gmtime().tm_hour <= 22:
        time.sleep(duration)


P = ParamSpec("P")
R = TypeVar("R")


def cached_json(filename: str) -> Callable[[Callable[P, R]], Callable[P, R]]:
    """
    Decorator which caches the functions' returned results in json format

    :filename: where to store the file
    """

    def decorator(func: Callable[[], R]) -> Callable[P, R]:  # needed, as we want to pass filename to the annotation
        decorator_filename: str = filename  # needed, as otherwise this context would be lost

        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            # prepare the filepath
            wrapper_filename = decorator_filename
            if args or kwargs:
                wrapper_filename = decorator_filename.format(*args, **kwargs)
            path = CACHE_PATH / wrapper_filename
            # get already existing file
            if path.exists():
                with open(path, encoding="utf-8") as file:
                    return json.load(file)  # type: ignore
            # produce new file
            result = func(*args, **kwargs)
            with open(CACHE_PATH / wrapper_filename, "w", encoding="utf-8") as file:
                json.dump(result, file, indent=2, sort_keys=True)
            return result

        return wrapper

    return decorator


def _download_file(url: str, target_cache_file: Path, quiet: bool = False, quiet_errors: bool = False) -> Path | None:
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
