import os
import itertools

import yaml

from utils import convert_to_webp

KNOWN_LICENSE_URLS = {
    "CC0": "https://creativecommons.org/publicdomain/zero/1.0/deed.en",
    "CC-BY 2.0": "https://creativecommons.org/licenses/by/2.0/deed.en",
    "CC-BY 2.5": "https://creativecommons.org/licenses/by/2.5/deed.en",
    "CC-BY 3.0": "https://creativecommons.org/licenses/by/3.0/deed.en",
    "CC-BY 4.0": "https://creativecommons.org/licenses/by/4.0/deed.en",
    "CC-BY-SA 2.0": "https://creativecommons.org/licenses/by-sa/2.0/deed.en",
    "CC-BY-SA 2.5": "https://creativecommons.org/licenses/by-sa/2.5/deed.en",
    "CC-BY-SA 3.0": "https://creativecommons.org/licenses/by-sa/3.0/deed.en",
    "CC-BY-SA 4.0": "https://creativecommons.org/licenses/by-sa/4.0/deed.en",
}


def add_img(data, path_prefix):
    """
    Automatially add processed images to the 'img' property.
    """
    with open(os.path.join(path_prefix, "img-sources.yaml")) as f:
        img_sources = yaml.safe_load(f.read())

    files = {
        "large": os.listdir(os.path.join(path_prefix, "large")),
        "header-small": os.listdir(os.path.join(path_prefix, "header-small")),
        "thumb": os.listdir(os.path.join(path_prefix, "thumb")),
    }

    # Check that all images have source information (to make sure it was not forgot)
    merged_filelist = list(itertools.chain(*files.values()))
    for f in merged_filelist:
        if ".webp" not in f:
            f = convert_to_webp(f)
        parts = f.lower().replace(".webp", "").split("_")
        try:
            _id = parts[0]
            _index = int(parts[1])
        except:
            print(f"Error: failed to parse image file name '{f}'")
            exit(1)

        if _id not in img_sources or _index not in img_sources[_id]:
            print(f"Warning: No source information for image '{f}', it will not be used")

    for _id, _source_data in img_sources.items():
        if _id not in data:
            print(f"Warning: There are images for '{_id}', but it was not found in the provided data, ignoring")
            continue

        matching_images = {
            subdir: list(filter(lambda f: f.startswith(_id + "_"), filelist))
            for subdir, filelist in files.items()
        }

        img_data = {}
        if len(matching_images["thumb"]) > 0:
            img_data["thumb"] = matching_images["thumb"][0]
        if len(matching_images["header-small"]) > 0:
            img_data["header_small"] = _add_source_info(matching_images["header-small"][0], _source_data)
        if len(matching_images["large"]) > 0:
            img_data["large"] = [
                _add_source_info(f, _source_data) for f in matching_images["large"]
            ]

        data[_id]["img"] = img_data


def _add_source_info(fname, source_data):
    if ".webp" not in fname:
        fname = convert_to_webp(fname)
    parts = fname.lower().replace(".webp", "").split("_")
    _id = parts[0]
    _index = int(parts[1])

    def _parse(obj):
        if type(obj) is str:
            return {"text": obj, "url": None}
        else:
            return obj

    img_data = {
        "name": fname,
        "author": _parse(source_data[_index]["author"])
    }
    if "source" in source_data[_index]:
        img_data["source"] = _parse(source_data[_index]["source"])
    if "license" in source_data[_index]:
        img_data["license"] = _parse(source_data[_index]["license"])
        if img_data["license"]["text"] in KNOWN_LICENSE_URLS:
            img_data["license"]["url"] = KNOWN_LICENSE_URLS[img_data["license"]["text"]]
        else:
            print(f"Warning: Unknown license url for '{img_data['license']['text']}'")

    return img_data




