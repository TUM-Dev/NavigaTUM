import hashlib
import json
import logging
import os
import shutil
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import Any, NamedTuple, TypeVar

import pydantic
import yaml
from PIL import Image
from pydantic import Field
from pydantic.networks import HttpUrl

import utils
from external.models.common import PydanticConfiguration


class UrlStr(PydanticConfiguration):
    text: str
    url: HttpUrl | None = None


class ImageOffset(PydanticConfiguration):
    header: int = 0
    thumb: int = 0


# here until typing.Self can be used in all expected python versions of developers
# pylint: disable-next=c-extension-no-member
# pylint: disable-next=invalid-name
TImageSource = TypeVar("TImageSource", bound="ImageSource")


class ImageSource(PydanticConfiguration):
    author: str
    license: UrlStr
    source: UrlStr
    meta: dict[str, str | pydantic.types.date] = Field(default_factory=dict)
    offsets: ImageOffset = Field(default_factory=ImageOffset)

    @classmethod
    def load_all(cls: TImageSource) -> dict[str, list[TImageSource]]:
        """Load the image sources from the img-sources.yaml file"""
        with open(IMAGE_BASE / "img-sources.yaml", encoding="utf-8") as file:
            raw: dict[str, dict[int, dict]] = yaml.safe_load(file.read())
            image_sources = {k: [ImageSource(**v) for v in vs.values()] for k, vs in raw.items()}
        for key in image_sources:
            if not isinstance(key, str):
                raise ValueError(
                    f"Key '{key}' form `img-sources.yaml` is not a string. "
                    "This is not allowed, as for integers leading zeros are silently ignored.",
                )
        return image_sources


IMAGE_BASE = Path(__file__).parent.parent / "sources" / "img"
IMAGE_SOURCE = IMAGE_BASE / "lg"
HASH_LUT = Path(IMAGE_BASE / ".hash_lut.json")

DEV_MODE = "GIT_COMMIT_SHA" not in os.environ
TARGET_IMAGE_QUALITY = 80


def add_img(data: dict[str, dict[str, Any]]) -> None:
    """Automatically add processed images to the 'img' property."""
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
            if image_path.name.startswith(f"{_id}_"):
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
    """Parse the filename of an image to get the id and index"""
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
        return {"text": obj, "url": None} if isinstance(obj, str) else obj

    return {
        "name": fname,
        "author": _parse(source_data[_index]["author"]),
        "source": _parse(source_data[_index]["source"]),
        "license": _parse(source_data[_index]["license"]),
    }


class Resizer:
    def __init__(self, source: Path):
        self.source = source
        self.img = Image.open(source)

    def resize_to_fixed_size(self, target: Path, fixed_size: tuple[int, int], offset: int) -> None:
        """
        Generate an image with fixed_size pixels for the given image.

        An offset can be used, to translate the image across the longer axis.
        """
        width, height = self.img.size
        mid_h = height // 2
        mid_w = width // 2

        target_w, target_h = fixed_size
        target_aspect_ratio = target_w / target_h
        current_aspect_ratio = width / height
        if target_aspect_ratio < current_aspect_ratio:
            # current image is wider than target, so we need to crop the width
            new_width = target_aspect_ratio * height
            new_img = self.img.crop(
                (mid_w - int(new_width / 2) + offset, 0, mid_w + int(new_width / 2) + offset, height),
            )
        elif target_aspect_ratio > current_aspect_ratio:
            # current image is higher than target, so we need to crop the height
            new_height = (1 / target_aspect_ratio) * width
            new_img = self.img.crop(
                (0, mid_h - int(new_height / 2) + offset, width, mid_h + int(new_height / 2) + offset),
            )
        else:
            # aspect ratio is the same, so no need to crop
            new_img = self.img.copy()
        # thumbnail may be more efficient, but does only handle square images
        if target_w == target_h:
            new_img.thumbnail(fixed_size, Image.Resampling.LANCZOS)
        else:
            new_img = new_img.resize(fixed_size, Image.Resampling.LANCZOS)
        new_img.save(target, lossless=False, quality=TARGET_IMAGE_QUALITY)

    def resize_to_max_size(self, target: Path, max_size: int) -> None:
        """Generate an image with at max_size pixel in max(width, height) for the given image."""
        width, height = self.img.size
        if max_size >= max(width, height):
            if target != self.source:
                # since we are already smaller than the max_size, we can copy the original image.
                if target.is_file():
                    os.remove(target)
                shutil.copy(self.source, target)
            return
        if width < height:
            # image is vertical
            scaling = max_size / height
            image = self.img.resize((int(width * scaling), max_size), Image.Resampling.LANCZOS)  # type: ignore
        else:
            # image is horizontal
            scaling = max_size / width
            image = self.img.resize((max_size, int(height * scaling)), Image.Resampling.LANCZOS)  # type: ignore
        image.save(target, lossless=False, quality=TARGET_IMAGE_QUALITY)


class RefreshResolutionOrder(NamedTuple):
    source: Path
    offsets: ImageOffset


def _refresh_for_all_resolutions(order: RefreshResolutionOrder) -> None:
    """Refresh an image for all resolutions"""
    try:
        resizer = Resizer(order.source)
        resizer.resize_to_max_size(IMAGE_BASE / "sm" / order.source.name, 1024)
        resizer.resize_to_max_size(IMAGE_BASE / "md" / order.source.name, 1920)
        resizer.resize_to_max_size(IMAGE_BASE / "lg" / order.source.name, 3840)
        resizer.resize_to_fixed_size(IMAGE_BASE / "thumb" / order.source.name, (256, 256), order.offsets.thumb)
        resizer.resize_to_fixed_size(IMAGE_BASE / "header" / order.source.name, (512, 210), order.offsets.header)
    # pylint: disable-next=broad-exception-caught
    except Exception as error:
        logging.error(error)  # otherwise we would not see if an error occurs


def _extract_offsets(_id: str, _index: int, img_path: Path, img_sources: dict[str, list[ImageSource]]) -> ImageOffset:
    """Extract the offsets for the given image. Offsets are only available for the images, we crop"""
    if _id not in img_sources or _index >= len(img_sources[_id]):
        logging.warning(f"No source information for image '{img_path}', default crop-offset 0 is used")
        return ImageOffset()
    return img_sources[_id][_index].offsets


def _get_hash_lut() -> dict[str, str]:
    """Get a lookup table for the hash of the image files content and offset if present"""
    logging.info("Only files, with sha256(file-content)_sha256(offset) not present in the .hash_lut.json will be used")
    if HASH_LUT.is_file():
        with open(HASH_LUT, encoding="utf-8") as file:
            return json.load(file)  # type: ignore
    return {}


def _save_hash_lut(img_sources: dict[str, list[ImageSource]]) -> None:
    """Save the current image status to the .hash_lut.json file"""
    hashes_lut = {}
    for img_path in IMAGE_SOURCE.glob("*.webp"):
        _id, _index = parse_image_filename(img_path.name)
        offsets = _extract_offsets(_id, _index, img_path, img_sources)
        hashes_lut[img_path.name] = _gen_file_hash(img_path, offsets)
    with open(HASH_LUT, "w+", encoding="utf-8") as file:
        json.dump(hashes_lut, file, sort_keys=True, indent=2)


def _gen_file_hash(img_path: Path, offsets: ImageOffset) -> str:
    """Generate a hash-string for the given image file and given offsets."""
    with open(img_path, "rb") as file:
        # pylint: disable-next=unexpected-keyword-arg
        file_hash = hashlib.sha256(file.read(), usedforsecurity=False).hexdigest()
        json_offsets = json.dumps({"thumb": offsets.thumb, "header": offsets.header}, sort_keys=True).encode("utf-8")
        # pylint: disable-next=unexpected-keyword-arg
        offset_hash = hashlib.sha256(json_offsets, usedforsecurity=False).hexdigest()
        return f"{file_hash}_{offset_hash}"


def resize_and_crop() -> None:
    """
    Resize and crop the images for the given data to the desired resolutions.

    This will overwrite any existing thumbs/header-small's.
    """
    logging.info(f"convert {IMAGE_BASE} to webp")
    utils.convert_to_webp(IMAGE_BASE)

    # in DEV, we can save some time by not resizing the images, if they have not changed
    expected_hashes_lut = _get_hash_lut()
    start_time = time.time()
    img_sources = ImageSource.load_all()
    with ThreadPoolExecutor() as executor:
        for img_path in IMAGE_SOURCE.glob("*.webp"):
            _id, _index = parse_image_filename(img_path.name)
            offsets = _extract_offsets(_id, _index, img_path, img_sources)
            actual_hash = _gen_file_hash(img_path, offsets)
            if actual_hash == expected_hashes_lut.get(img_path.name, ""):
                continue  # skip this image, since it (and its offsets) have not changed
            if DEV_MODE:
                logging.debug(f"Image '{img_path.name}' has changed, resizing and cropping...")
            executor.submit(_refresh_for_all_resolutions, RefreshResolutionOrder(img_path, offsets))
    _save_hash_lut(img_sources)
    resize_and_crop_time = time.time() - start_time
    logging.info(f"Resize and crop took {resize_and_crop_time:.2f}s")
