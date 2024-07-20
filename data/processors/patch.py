import logging
import re
from typing import TypedDict

from external.models import tumonline

# python 3.11 feature => move to typing when 3.11 is mainstream
from typing_extensions import NotRequired


class Patch(TypedDict):
    if_room_code: str
    alt_name: NotRequired[str]
    arch_name: NotRequired[str]


def apply_roomcode_patch(objects: dict[str, tumonline.Room], patches: list[Patch]):
    """
    Apply patches to objects.

    Args:
    ----
        objects: list of objects to apply patches to
        patches: list of patches to apply

    """
    patches = [
        (
            re.compile(p["if_room_code"]),
            {k: v for k, v in p.items() if k != "if_room_code"},
        )
        for p in patches
    ]

    to_delete = []
    applied_patches = set()
    for room_code, room in objects.items():
        for patch_check, patch in patches:
            if patch_check.match(room_code) is not None:
                applied_patches.add(patch_check)
                if patch.get("__delete"):
                    to_delete.append(room_code)
                    continue
                for patch_key, patched_value in patch.items():
                    setattr(room, patch_key, patched_value)
                    room.patched = True

    for room_code in to_delete:
        objects.pop(room_code)

    for patch_check, _ in patches:
        if patch_check not in applied_patches:
            logging.warning(
                f"The patch for roomcode: r'{patch_check.pattern}' was never applied. "
                f"Make sure it is still required.",
            )
