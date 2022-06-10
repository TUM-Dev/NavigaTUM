import hashlib
import json
import logging
import os
import shutil
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Any, Optional

import yaml
from PIL import Image

IMAGE_BASE = Path(__file__).parent.parent / "sources" / "img"
IMAGE_SOURCE = IMAGE_BASE / "lg"
HASH_LUT = Path(IMAGE_BASE / ".hash_lut.json")

DEV_MODE = "GIT_COMMIT_SHA" not in os.environ

RESOLUTIONS: list[tuple[str, int | tuple[int, int]]] = [
    ("thumb", (256, 256)),
    ("header", (512, 210)),
    ("sm", 1024),  # max. 1024px
    ("md", 1920),  # max. 1920px
    ("lg", 3840),  # max. 4k, this is the source
]

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


def add_img(data):
    """
    Automatically add processed images to the 'img' property.
    """
    with open(IMAGE_BASE / "img-sources.yaml", encoding="utf-8") as file:
        img_sources = yaml.safe_load(file.read())

    # Check that all images have source information (to make sure it was not forgotten)
    for image_path in IMAGE_SOURCE.iterdir():
        _id, _index = parse_image_filename(image_path.name)

        if _id not in img_sources or _index not in img_sources[_id]:
            logging.warning(f"No source information for image '{image_path}', it will not be used")

    # filter the images, that should exist, by the ones that actually do
    for _id, _source_data in img_sources.items():
        if _id not in data:
            logging.warning(f"There are images for '{_id}', but it was not found in the provided data, ignoring")
            continue

        img_data = []
        for image_path in IMAGE_SOURCE.iterdir():
            if image_path.name.startswith(_id + "_"):
                source_info = _add_source_info(image_path.name, _source_data)
                if not source_info:
                    logging.warning(
                        f"possibly skipped adding images for '{image_path}', a image was skipped because of missing "
                        f"source information. Adding more images would violate the enumeration-consistency",
                    )
                    break
                img_data.append(source_info)

        data[_id]["imgs"] = img_data


def parse_image_filename(image_name: str) -> tuple[str, int]:
    """parse the filename of an image to get the id and index"""
    if ".webp" not in image_name:
        raise RuntimeError(f"Missing webp for '{image_name}'")
    parts = image_name.replace(".webp", "").split("_")
    try:
        _id = parts[0]
        _index = int(parts[1])
        return _id, _index
    except Exception as error:
        raise RuntimeError(f"Error: failed to parse image file name '{image_name}'") from error


def _add_source_info(fname, source_data):
    _id, _index = parse_image_filename(fname)

    required_fields = ["author", "license", "source"]
    for field in required_fields:
        if field not in source_data[_index]:
            logging.warning(f"No {field} information for image '{fname}', it will not be used")
            return None

    def _parse(obj):
        if isinstance(obj, str):
            return {"text": obj, "url": None}
        return obj

    img_data = {
        "name": fname,
        "author": _parse(source_data[_index]["author"]),
        "source": _parse(source_data[_index]["source"]),
    }
    # add license information utilising shorthands if available
    raw_license = source_data[_index]["license"]
    if isinstance(raw_license, str):
        if raw_license in KNOWN_LICENSE_URLS:
            img_data["license"] = {"text": raw_license, "url": KNOWN_LICENSE_URLS[raw_license]}
            return img_data
        logging.warning(f"Unknown license url for licence shorthand '{raw_license}'. No url will be added")
    img_data["license"] = _parse(raw_license)
    return img_data


def _gen_fixed_size(img: Image.Image, fixed_size: tuple[int, int], offset: int) -> Image.Image:
    """
    Generate an image with fixed_size pixels for the given image.
    An offset can be used, to translate the image across the longer axis.
    """
    width, height = img.size
    mid_h = height // 2
    mid_w = width // 2

    target_w, target_h = fixed_size
    target_aspect_ratio = target_w / target_h
    current_aspect_ratio = width / height
    if target_aspect_ratio < current_aspect_ratio:
        # current image is wider than target, so we need to crop the width
        new_width = target_aspect_ratio * height
        new_img = img.crop((mid_w - int(new_width / 2) + offset, 0, mid_w + int(new_width / 2) + offset, height))
    elif target_aspect_ratio > current_aspect_ratio:
        # current image is higher than target, so we need to crop the height
        new_height = (1 / target_aspect_ratio) * width
        new_img = img.crop((0, mid_h - int(new_height / 2) + offset, width, mid_h + int(new_height / 2) + offset))
    else:
        # aspect ratio is the same, so no need to crop
        new_img = img.copy()
    if target_w != target_h:
        # thumbnail may be more efficient, but does only handle square images
        return new_img.resize(fixed_size, Image.Resampling.LANCZOS)  # type: ignore
    new_img.thumbnail(fixed_size, Image.Resampling.LANCZOS)  # type: ignore
    return new_img


def _gen_max_size(img: Image.Image, max_size: int) -> Optional[Image.Image]:
    """Generate an image with at max_size pixel in max(width, height) for the given image."""
    width, height = img.size
    if max(width, height) <= max_size:
        # since we are already smaller than the max_size, we can copy the original image.
        # To indicate this we return None
        return None
    if width < height:
        # image is vertical
        scaling = max_size / height
        return img.resize((int(width * scaling), max_size), Image.Resampling.LANCZOS)  # type: ignore
    # image is horizontal
    scaling = max_size / width
    return img.resize((max_size, int(height * scaling)), Image.Resampling.LANCZOS)  # type: ignore


def _refresh_for_all_resolutions(args: tuple[Path, dict[str, int]]) -> None:
    source_filepath, offsets = args
    img: Image.Image = Image.open(source_filepath)
    for target_dir_name, size in RESOLUTIONS:
        target_filepath = IMAGE_BASE / target_dir_name / source_filepath.name
        if isinstance(size, int):
            new_img = _gen_max_size(img, size)
            if new_img is None:  # we are already smaller than the max_size, so we can copy the original image
                if source_filepath == target_filepath:
                    continue
                if target_filepath.is_file():
                    os.remove(target_filepath)
                shutil.copy(source_filepath, target_filepath)
                continue
        else:
            new_img = _gen_fixed_size(img, size, offsets.get(target_dir_name, 0))
        new_img.save(target_filepath, lossless=False, quality=50)


def _extract_offsets(_id: str, _index: int, img_path: Path, img_sources: Any) -> dict:
    """Extract the offsets for the given image. Offsets are only available for the images, we crop"""
    for _target_dir_name, size in RESOLUTIONS:
        if isinstance(size, tuple):
            if _id in img_sources and _index in img_sources[_id]:
                return img_sources[_id][_index].get("offsets", {})
            logging.warning(f"No source information for image '{img_path}', default crop-offset 0 is used")
    return {}


def _get_hash_lut() -> dict[str, str]:
    """Get a lookup table for the hash of the image files content and offset if present"""
    logging.info("Since GIT_COMMIT_SHA is unset, we assume this is acting in In Dev mode.")
    logging.info("Only files, with sha256(file-content)_sha256(offset) not present in the .hash_lut.json will be used")
    if HASH_LUT.is_file():
        with open(HASH_LUT, encoding="utf-8") as file:
            return json.load(file)
    return {}


def _save_hash_lut(img_sources) -> None:
    """Save the current image status to the .hash_lut.json file"""
    hashes_lut = {}
    for img_path in IMAGE_SOURCE.glob("*.webp"):
        _id, _index = parse_image_filename(img_path.name)
        offsets = _extract_offsets(_id, _index, img_path, img_sources)
        hashes_lut[img_path.name] = _gen_file_hash(img_path, offsets)
    with open(HASH_LUT, "w+", encoding="utf-8") as file:
        json.dump(hashes_lut, file, sort_keys=True, indent=4)


def _gen_file_hash(img_path: Path, offsets) -> str:
    """Generate a hash-string for the given image file and given offsets."""
    with open(img_path, "rb") as file:
        # pylint: disable-next=unexpected-keyword-arg
        file_hash = hashlib.sha256(file.read(), usedforsecurity=False).hexdigest()
        json_offsets = json.dumps(offsets, sort_keys=True).encode("utf-8")
        # pylint: disable-next=unexpected-keyword-arg
        offset_hash = hashlib.sha256(json_offsets, usedforsecurity=False).hexdigest()
        return f"{file_hash}_{offset_hash}"


def resize_and_crop() -> None:
    """
    Resize and crop the images for the given data to the desired resolutions.
    This will overwrite any existing thumbs/header-small's.
    """
    for target_dir_name, _size in RESOLUTIONS:
        target_dir = IMAGE_BASE / target_dir_name
        if not target_dir.exists():
            target_dir.mkdir()
    # in DEV, we can save some time by not resizing the images, if they have not changed
    expected_hashes_lut = {}
    if DEV_MODE:
        expected_hashes_lut = _get_hash_lut()
    start_time = time.time()
    with open(IMAGE_BASE / "img-sources.yaml", encoding="utf-8") as file:
        img_sources = yaml.safe_load(file.read())
    with ThreadPoolExecutor() as executor:
        for img_path in IMAGE_SOURCE.glob("*.webp"):
            _id, _index = parse_image_filename(img_path.name)
            offsets = _extract_offsets(_id, _index, img_path, img_sources)
            if DEV_MODE:
                actual_hash = _gen_file_hash(img_path, offsets)
                if actual_hash == expected_hashes_lut.get(img_path.name, ""):
                    continue  # skip this image, since it (and its offsets) have not changed
                logging.info(f"Image '{img_path.name}' has changed, resizing and cropping...")
            executor.submit(_refresh_for_all_resolutions, (img_path, offsets))
    resize_and_crop_time = time.time() - start_time
    if DEV_MODE:
        _save_hash_lut(img_sources)
    logging.info(f"Resize and crop took {resize_and_crop_time:.2f}s")
