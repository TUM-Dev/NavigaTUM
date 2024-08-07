import time
from pathlib import Path

import backoff
import requests

CACHE_PATH = Path(__file__).parent / "results"


def maybe_sleep(duration: float) -> None:
    """Sleep for the given duration, but only if the script was called during a workday and working hours."""
    if time.gmtime().tm_wday not in [5, 6] and 7 <= time.gmtime().tm_hour <= 20:
        time.sleep(duration)


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def _download_file(url: str, target_cache_file: Path) -> Path | None:
    if target_cache_file.exists():
        target_cache_file.unlink()
    # url parameter does not allow path traversal, because we build it further up in the callstack
    with requests.get(url, stream=True, timeout=10) as r:
        r.raise_for_status()
        with target_cache_file.open("wb") as f:
            for chunk in r.iter_content(chunk_size=8192):
                f.write(chunk)
    return target_cache_file
