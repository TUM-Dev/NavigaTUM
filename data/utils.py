import logging
import os
from math import acos, cos, radians, sin
from pathlib import Path
from typing import Any, Union

from PIL import Image
from ruamel.yaml import YAML

yaml = YAML(typ="rt")

TRANSLATION_BUFFER_PATH = Path(__file__).parent / "translations.yaml"

with TRANSLATION_BUFFER_PATH.open(encoding="utf-8") as yaml_file:
    TRANSLATION_BUFFER = yaml.load(yaml_file)

DEV_MODE = "GIT_COMMIT_SHA" not in os.environ


class TranslatableStr(dict):
    """
    Wrapper for translations.

    The Wrapper takes a string, that should be translated and looks it up in the translation buffer.
    If the string is not found, it an entry is created in the buffer.

    The string can be formatted using the .format() method or left as is.

    Since it is a subclass of dict, this class does not need any additional infrastructure
    to turn a message into a translated string.

    Translatable strings will be exported as {"de": "<de string>", "en": "<en string>"}.
    """

    def __init__(self, message: str, en_message: str | None = None) -> None:
        if not isinstance(message, str):
            raise ValueError("message must be a str")
        if not message.strip():
            raise ValueError("message must not be empty")
        if en_message is None:
            if message in TRANSLATION_BUFFER:
                en_message = TRANSLATION_BUFFER[message]
            else:
                en_message = message
                TRANSLATION_BUFFER[message] = ""
                with TRANSLATION_BUFFER_PATH.open("w", encoding="utf-8") as file:
                    yaml.dump(TRANSLATION_BUFFER, file)
        super().__init__(en=en_message, de=message)

    def __hash__(self) -> int:
        """Return a hash as if this was a string."""
        return hash(self["de"])

    def __le__(self, other: "TranslatableStr") -> bool:
        """Compare one String to another, sorting by the german string."""
        return self["de"] <= other["de"]

    def __lt__(self, other: "TranslatableStr") -> bool:
        """Compare one String to another, sorting by the german string."""
        return self["de"] < other["de"]

    def __add__(self, other: Union[str, "TranslatableStr"]) -> "TranslatableStr":
        """Concatenate two TranslatableStr or a TranslatableStr with a string ."""
        if isinstance(other, str):
            return TranslatableStr(self["de"] + other, self["en"] + other)
        if isinstance(other, TranslatableStr):
            return TranslatableStr(self["de"] + other["de"], self["en"] + other["en"])
        raise ValueError(f"{self} + {other} is not implmented")

    def __radd__(self, other: str) -> "TranslatableStr":
        """Concatenate a TranslatableStr onto a string"""
        if isinstance(other, str):
            return TranslatableStr(other + self["de"], other + self["en"])
        raise ValueError(f"{other} + {self} is not implmented")

    def format(self, *args: Any, **kwargs: Any) -> "TranslatableStr":
        """Apply the format-method to the contained data, as if the class itsself was a string."""
        self["de"] = self["de"].format(*args, **kwargs)
        self["en"] = self["en"].format(*args, **kwargs)
        return self


def convert_to_webp(source: Path) -> None:
    """
    Convert image(s) to WebP.

    Args:
    ----
        source (pathlib.Path): Path to source image(s)

    Returns:
    -------
        pathlib.Path: path to new image(s)

    """
    if source.is_dir():
        for img_path in source.iterdir():
            if img_path.suffix not in [".webp", ".yaml", ".json"] and img_path.name != ".gitkeep":
                convert_to_webp(img_path)
        return

    destination = source.with_suffix(".webp")

    image = Image.open(source)
    image.save(destination, format="webp")
    os.remove(source)


def setup_logging(level: int = logging.INFO) -> None:
    """Set up the loglevels with colors, with correct terminal colors"""
    logging.basicConfig(level=level, format="%(levelname)s: %(message)s")
    logging.addLevelName(logging.INFO, f"\033[1;36m{logging.getLevelName(logging.INFO)}\033[1;0m")
    logging.addLevelName(logging.WARNING, f"\033[1;33m{logging.getLevelName(logging.WARNING)}\033[1;0m")
    logging.addLevelName(logging.ERROR, f"\033[1;41m{logging.getLevelName(logging.ERROR)}\033[1;0m")
    logging.addLevelName(logging.CRITICAL, f"\033[1;41m{logging.getLevelName(logging.CRITICAL)}\033[1;0m")


EARTH_RADIUS_METERS: int = 6_371_000


def distance_via_great_circle(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """
    Calculate the approximate distance in meters between two points using the great circle approach

    Basic idea from https://blog.petehouston.com/calculate-distance-of-two-locations-on-earth/
    """
    if lat1 == lat2 and lon1 == lon2:
        return 0.0
    lat1, lon1 = radians(lat1), radians(lon1)
    lat2, lon2 = radians(lat2), radians(lon2)

    # angular distance using the https://wikipedia.org/wiki/Haversine_formula
    angular_distance = acos(sin(lat1) * sin(lat2) + cos(lat1) * cos(lat2) * cos(lon1 - lon2))
    return EARTH_RADIUS_METERS * angular_distance
