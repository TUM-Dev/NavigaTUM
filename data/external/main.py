import logging
import os

from external.scrapers import nat, roomfinder, tumonline
from external.scraping_utils import CACHE_PATH
from utils import setup_logging

if __name__ == "__main__":
    setup_logging(level=logging.INFO)
    # Create cache directory structure
    os.makedirs(CACHE_PATH, exist_ok=True)
    os.makedirs(CACHE_PATH / "filter", exist_ok=True)
    os.makedirs(CACHE_PATH / "tumonline", exist_ok=True)
    os.makedirs(CACHE_PATH / "nat", exist_ok=True)
    os.makedirs(CACHE_PATH / "room", exist_ok=True)
    os.makedirs(CACHE_PATH / "maps" / "roomfinder", exist_ok=True)
    os.makedirs(CACHE_PATH / "maps" / "roomfinder" / "kmz", exist_ok=True)

    # You can comment out steps that should be skipped.
    # The downloader will automatically create a cache in `cache/`.
    roomfinder.scrape_buildings()
    tumonline.scrape_buildings()
    nat.scrape_buildings()

    roomfinder.scrape_rooms()
    tumonline.scrape_rooms()
    nat.scrape_rooms()

    tumonline.scrape_usages()

    roomfinder.scrape_maps()

    tumonline.scrape_orgs(lang="de")
    tumonline.scrape_orgs(lang="en")
