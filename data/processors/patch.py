import logging
import re


def apply_patches(objects, patches, searchkey):
    patched = []

    patches = [
        (
            re.compile(p["if_" + searchkey]),
            # Remove the "if_" from the patch, the rest of the items will
            # be inserted into the entry's data.
            dict(filter(lambda e: e[0] != "if_" + searchkey, p.items())),
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
                else:
                    for k, v in patch.items():
                        obj[k] = v
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
