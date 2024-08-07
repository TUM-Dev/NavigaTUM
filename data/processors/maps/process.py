from typing import Any

import utils
from processors.maps.models import Overlay
from processors.maps.overlay import add_overlay_map
from processors.maps.roomfinder import (
    assign_default_roomfinder_map,
    assign_roomfinder_maps,
    build_roomfinder_maps,
    CUSTOM_RF_DIR_PATH,
    remove_non_covering_maps,
)


def add_overlay_maps(data: dict[str, dict[str, Any]]) -> None:
    """Add the overlay maps to all entries where they apply"""
    parent_lut = Overlay.load_all()
    parent_ids = set(parent_lut.keys())

    for _id, entry in data.items():
        add_overlay_map(_id, entry, parent_ids, parent_lut)


def add_roomfinder_maps(data: dict[str, dict[str, Any]]) -> None:
    """Add roomfinder maps to entries"""
    utils.convert_to_webp(CUSTOM_RF_DIR_PATH)

    assign_roomfinder_maps(data)
    remove_non_covering_maps(data)
    assign_default_roomfinder_map(data)
    build_roomfinder_maps(data)
