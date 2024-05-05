import logging
import os
from pathlib import Path

from external.scrapers import nat, public_transport, roomfinder, tumonline
from external.scraping_utils import CACHE_PATH
from utils import setup_logging


def ensure_is_empty(directory: Path):
    """
    Make the specified directory empty by recursively deleting all its contents.

    Args:
    ----
        directory: The directory path to be emptied.

    Returns:
    -------
        None

    """
    for item in directory.iterdir():
        if item.is_dir():
            try:
                os.removedirs(item)
            except OSError:
                ensure_is_empty(item)
        else:
            item.unlink()
    directory.rmdir()
    os.makedirs(CACHE_PATH, exist_ok=True)


if __name__ == "__main__":
    setup_logging(level=logging.INFO)
    ensure_is_empty(CACHE_PATH)

    tumonline.scrape_buildings()
    tumonline.scrape_rooms()
    tumonline.scrape_usages()
    tumonline.scrape_orgs(lang="de")
    tumonline.scrape_orgs(lang="en")

    os.makedirs(CACHE_PATH / "nat", exist_ok=True)
    nat.scrape_buildings()
    nat.scrape_rooms()

    os.makedirs(CACHE_PATH / "maps" / "roomfinder", exist_ok=True)
    os.makedirs(CACHE_PATH / "maps" / "roomfinder" / "kmz", exist_ok=True)
    roomfinder.scrape_buildings()
    roomfinder.scrape_rooms()
    roomfinder.scrape_maps()

    os.makedirs(CACHE_PATH / "public_transport", exist_ok=True)
    public_transport.scrape_stations()
