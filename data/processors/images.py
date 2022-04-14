import itertools
import os
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import yaml
from PIL import Image
from utils import convert_to_webp

KNOWN_LICENSE_URLS = {
    "CC0 1.0": "https://creativecommons.org/publicdomain/zero/1.0/deed.en",
    "CC-BY 2.0": "https://creativecommons.org/licenses/by/2.0/deed.en",
    "CC-BY 2.5": "https://creativecommons.org/licenses/by/2.5/deed.en",
    "CC-BY 3.0": "https://creativecommons.org/licenses/by/3.0/deed.en",
    "CC-BY 4.0": "https://creativecommons.org/licenses/by/4.0/deed.en",
    "CC-BY-SA 2.0": "https://creativecommons.org/licenses/by-sa/2.0/deed.en",
    "CC-BY-SA 2.5": "https://creativecommons.org/licenses/by-sa/2.5/deed.en",
    "CC-BY-SA 3.0": "https://creativecommons.org/licenses/by-sa/3.0/deed.en",
    "CC-BY-SA 4.0": "https://creativecommons.org/licenses/by-sa/4.0/deed.en",
}
THUMBNAIL_SIZE = (256, 256)
HEADER_MAX_SIZE = 1920


def add_img(data, path_prefix):
    """
    Automatially add processed images to the 'img' property.
    """
    with open(os.path.join(path_prefix, "img-sources.yaml")) as f:
        img_sources = yaml.safe_load(f.read())

    convert_to_webp(Path(path_prefix))

    files = {
        "large": os.listdir(os.path.join(path_prefix, "large")),
        "header-small": os.listdir(os.path.join(path_prefix, "header-small")),
        "thumb": os.listdir(os.path.join(path_prefix, "thumb")),
    }

    # Check that all images have source information (to make sure it was not forgot)
    merged_filelist = list(itertools.chain(*files.values()))
    for f in merged_filelist:
        _id, _index = parse_image_filename(f)

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


def parse_image_filename(f: str) -> tuple[str, int]:
    """parse the filename of an image to get the id and index"""
    if ".webp" not in f:
        raise RuntimeError(f"Missing webp for '{f}'")
    parts = f.replace(".webp", "").split("_")
    try:
        _id = parts[0]
        _index = int(parts[1])
        return _id, _index
    except Exception as e:
        raise RuntimeError(f"Error: failed to parse image file name '{f}'") from e


def _add_source_info(fname, source_data):
    if ".webp" not in fname:
        fname = convert_to_webp(Path(fname))
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
        if img_data["license"]["url"] is None:
            if img_data["license"]["text"] in KNOWN_LICENSE_URLS:
                img_data["license"]["url"] = KNOWN_LICENSE_URLS[img_data["license"]["text"]]
            else:
                print(f"Warning: Unknown license url for '{img_data['license']['text']}'")

    return img_data


def _gen_thumb(img: Image, base_dir: Path, filename: str, thumbnail_offset: int) -> None:
    """Generate a thumbnail for the given image."""
    w, h = img.size
    mid_h = h // 2
    mid_w = w // 2
    if w < h:
        # image is vertical
        thumb = img.crop((0, mid_h - mid_w + thumbnail_offset, w, mid_h + mid_w + thumbnail_offset))
    elif w > h:
        # image is horizontal
        thumb = img.crop((mid_w - mid_h + thumbnail_offset, 0, mid_w + mid_h + thumbnail_offset, h))
    else:
        # image is already square
        thumb = img
    thumb.thumbnail(THUMBNAIL_SIZE)
    thumb.save(base_dir / "thumb" / filename, lossless=False, method=6, quality=50)


def _gen_header(img: Image, base_dir: Path, filename: str) -> None:
    """Generate a header-small for the given image."""
    w, h = img.size
    header = img
    if max(w, h) > HEADER_MAX_SIZE:
        if w < h:
            # image is vertical
            scaling = HEADER_MAX_SIZE / h
            header = img.resize((int(w * scaling), HEADER_MAX_SIZE), Image.ANTIALIAS)
        else:
            # image is horizontal
            scaling = HEADER_MAX_SIZE / w
            header = img.resize((HEADER_MAX_SIZE, int(h * scaling)), Image.ANTIALIAS)
    header.save(base_dir / "header-small" / filename, lossless=False, method=6, quality=50)


def refresh_headers_and_thumbs(path):
    """
    Refresh the headers and thumbs for the given data.
    This will overwrite any existing thumbs/header-small's.
    """
    base_dir = Path(path)
    large_files_dir = base_dir / "large"

    def _refresh_single_headers_and_thumbs(args: tuple[Path, int]) -> None:
        img_filepath, thumbnail_offset = args
        img = Image.open(img_filepath)
        img_base_dir = img_filepath.parent.parent
        filename = img_filepath.name
        _gen_thumb(img, img_base_dir, filename, thumbnail_offset)
        _gen_header(img, img_base_dir, filename)

    with open(base_dir / "img-sources.yaml") as f:
        img_sources = yaml.safe_load(f.read())
    with ThreadPoolExecutor() as executor:
        for img_path in large_files_dir.glob("*.webp"):
            _id, _index = parse_image_filename(img_path.name)

            offset = 0
            if _id in img_sources and _index in img_sources[_id]:
                if "thumbnail_offset" in img_sources[_id][_index]:
                    offset = img_sources[_id][_index]["thumbnail_offset"]
            else:
                print(f"Warning: No source information for image '{img_path.name}', defaulting thumbnail-crop-offset "
                      f"to the center of the image")
            executor.submit(_refresh_single_headers_and_thumbs, (img_path, offset))
