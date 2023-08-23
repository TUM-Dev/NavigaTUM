import logging
from typing import Any

from processors.maps.models import Overlay


def add_overlay_map(_id: str, entry: dict[str, Any], parent_ids: set[str], parent_lut: dict[str, Overlay]) -> None:
    """Add the overlay maps to all entries where they apply"""
    candidates = parent_ids.intersection(entry["parents"])
    if len(candidates) > 1:
        logging.warning(
            f"Multiple candidates as overlay map for {_id}: {candidates}. "
            f"Currently this is not supported! Skipping ...",
        )
    elif bool(candidates) ^ (_id in parent_ids):
        # either a candidate exist or _id is one of the parent ids, but not both
        overlay = parent_lut[list(candidates)[0] if len(candidates) == 1 else _id]
        overlay_data = entry.setdefault("maps", {}).setdefault("overlays", {})
        overlay_data["available"] = []
        for _map in overlay.maps:
            overlay_data["available"].append(
                {
                    "id": _map.floor_index,
                    "floor": _map.floor,
                    "file": _map.file,
                    "name": _map.desc,
                    "coordinates": overlay.props.box,
                },
            )

            # The 'tumonline' field overwrites which TUMonline ID floor to match
            if (f".{_map.tumonline or ''}." in _id) or (
                overlay_data.get("default", None) is None and f".{_map.floor}." in _id
            ):
                overlay_data["default"] = _map.floor_index

        overlay_data.setdefault("default", None)
