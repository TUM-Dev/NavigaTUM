import os

from external.scrapers import roomfinder, tumonline
from external.scraping_utils import CACHE_PATH

if __name__ == "__main__":
    # Create cache directory structure
    os.makedirs(CACHE_PATH, exist_ok=True)
    os.makedirs(CACHE_PATH / "filter", exist_ok=True)
    os.makedirs(CACHE_PATH / "tumonline", exist_ok=True)
    os.makedirs(CACHE_PATH / "room", exist_ok=True)
    os.makedirs(CACHE_PATH / "maps" / "roomfinder", exist_ok=True)
    os.makedirs(CACHE_PATH / "maps" / "roomfinder" / "kmz", exist_ok=True)

    # You can comment out steps that should be skipped.
    # The downloader will automatically create a cache in `cache/`.
    roomfinder.roomfinder_buildings()
    tumonline.tumonline_buildings()

    roomfinder.roomfinder_rooms()
    tumonline.tumonline_rooms()

    tumonline.tumonline_usages()

    roomfinder.roomfinder_maps()
    tumonline.tumonline_orgs()
