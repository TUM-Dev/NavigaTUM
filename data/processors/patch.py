import logging
import re


def apply_patches(objects, patches, searchkey):
    """Apply patches to objects.

    Args:
        objects: list of objects to apply patches to
        patches: list of patches to apply
        searchkey: key to search for in objects"""
    patched = []

    patches = [
        (
            re.compile(p["if_" + searchkey]),
            # Remove the "if_" from the patch, the rest of the items will
            # be inserted into the entry's data.
            {k: v for k, v in p.items() if k != "if_" + searchkey},
        )
        for p in patches
    ]

    to_delete = []
    applied_patches = set()
    for i, obj in enumerate(objects):
        for patch_check, patch in patches:
            if patch_check.match(obj[searchkey]) is not None:
                applied_patches.add(patch_check)
                if "__delete" in patch and patch["__delete"]:
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
                f"The patch for {searchkey}: r'{patch_check.pattern}' was never applied. "
                f"Make sure it is still required.",
            )

    return patched
