import logging
import re

# python 3.11 feature => move to typing when 3.11 is mainstream
from typing import NotRequired, TypedDict

from pipeline_types import Entry

_logger = logging.getLogger(__name__)


class Patch(TypedDict):
    if_room_code: str
    alt_name: NotRequired[str]
    arch_name: NotRequired[str]


def apply_roomcode_patch(objects: dict[str, Entry], patches: list[Patch]) -> None:
    """
    Apply patches to objects.

    Args:
    ----
        objects: list of objects to apply patches to
        patches: list of patches to apply

    """
    compiled_patches: list[tuple[re.Pattern[str], dict[str, object]]] = [
        (
            re.compile(p["if_room_code"]),
            {k: v for k, v in p.items() if k != "if_room_code"},
        )
        for p in patches
    ]

    to_delete = []
    applied_patches: set[re.Pattern[str]] = set()
    for room_code, room in objects.items():
        for patch_check, patch in compiled_patches:
            if patch_check.match(room_code) is not None:
                applied_patches.add(patch_check)
                if patch.get("__delete"):
                    to_delete.append(room_code)
                    continue
                for patch_key, patched_value in patch.items():
                    room[patch_key] = patched_value
                    room["patched"] = True

    for room_code in to_delete:
        objects.pop(room_code)

    for patch_check, _ in compiled_patches:
        if patch_check not in applied_patches:
            _logger.warning(
                f"The patch for roomcode: r'{patch_check.pattern}' was never applied. Make sure it is still required.",
            )
