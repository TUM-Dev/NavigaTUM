import logging
import re
from typing import TypedDict

# python 3.11 feature => move to typing when 3.11 is mainstream
from typing_extensions import NotRequired


class Patch(TypedDict):
    if_roomcode: str
    alt_name: NotRequired[str]
    arch_name: NotRequired[str]


def apply_roomcode_patch(objects: list[dict[str, str | int]], patches: list[Patch]):
    """
    Apply patches to objects.

    Args:
    ----
        objects: list of objects to apply patches to
        patches: list of patches to apply

    """
    patched = []

    patches = [
        (
            re.compile(p["if_roomcode"]),
            # Remove the "if_" from the patch, the rest of the items will
            # be inserted into the entry's data.
            {k: v for k, v in p.items() if k != "if_roomcode"},
        )
        for p in patches
    ]

    to_delete = []
    applied_patches = set()
    for i, obj in enumerate(objects):
        for patch_check, patch in patches:
            if patch_check.match(obj["roomcode"]) is not None:
                applied_patches.add(patch_check)
                if patch.get("__delete"):
                    to_delete.append(i)
                    continue
                for patch_key, patched_value in patch.items():
                    obj[patch_key] = patched_value
                patched.append(obj)

    for i in reversed(to_delete):
        del objects[i]

    for patch_check, _ in patches:
        if patch_check not in applied_patches:
            logging.warning(
                f"The patch for roomcode: r'{patch_check.pattern}' was never applied. "
                f"Make sure it is still required.",
            )

    return patched
