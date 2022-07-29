import os

import downloader

if __name__ == "__main__":
    # Create cache directory structure
    cache_base = os.path.join(os.path.dirname(__file__), "cache")
    os.makedirs(cache_base, exist_ok=True)
    os.makedirs(os.path.join(cache_base, "filter"), exist_ok=True)
    os.makedirs(os.path.join(cache_base, "tumonline"), exist_ok=True)
    os.makedirs(os.path.join(cache_base, "room"), exist_ok=True)
    os.makedirs(os.path.join(cache_base, "maps/roomfinder"), exist_ok=True)
    os.makedirs(os.path.join(cache_base, "maps/roomfinder/kmz"), exist_ok=True)

    # You can comment out steps that should be skipped.
    # The downloader will automatically create a cache in `cache/`.
    downloader.roomfinder_buildings()
    downloader.tumonline_buildings()

    downloader.roomfinder_rooms()
    downloader.tumonline_rooms()

    downloader.tumonline_usages()

    downloader.roomfinder_maps()
    downloader.tumonline_orgs()
