import os
from pathlib import Path

from PIL import Image
from ruamel.yaml import YAML

yaml = YAML(typ="rt")

TRANSLATION_BUFFER_PATH = Path(__file__).parent / "translations.yaml"

with open(TRANSLATION_BUFFER_PATH, encoding="utf-8") as yaml_file:
    TRANSLATION_BUFFER = yaml.load(yaml_file)


class TranslatableStr(dict):
    """
    Wrapper for translations.
    Takes a string, that should be translated and looks it up in the translation buffer.
    If the string is not found, it an entry is created in the buffer.

    The string can be formatted using the .format() method or left as is.

    since subclassing from dict, this class does not need any aditional infrasturcture
    to turn a message into a translated string.
    """

    def __init__(self, message):
        if message in TRANSLATION_BUFFER:
            en_message = TRANSLATION_BUFFER[message]
        else:
            en_message = message
            TRANSLATION_BUFFER[message] = ""
            with open(TRANSLATION_BUFFER_PATH, "w", encoding="utf-8") as file:
                yaml.dump(TRANSLATION_BUFFER, file)
        super().__init__(en=en_message, de=message)

    def __hash__(self):
        """returns a hash as if this was a string."""
        return hash(self["de"])

    def format(self, *args, **kwargs):
        """Format the string using the .format() method, as if this was a string."""
        self["de"] = self["de"].format(*args, **kwargs)
        self["en"] = self["en"].format(*args, **kwargs)
        return self


def convert_to_webp(source: Path):
    """Convert image(s) to WebP.

    Args:
        source (pathlib.Path): Path to source image(s)

    Returns:
        pathlib.Path: path to new image(s)
    """
    if source.is_dir():
        for img_path in source.iterdir():
            if img_path.suffix not in [".webp", ".yaml", ".json"]:
                convert_to_webp(img_path)
        return source

    destination = source.with_suffix(".webp")

    image = Image.open(source)
    image.save(destination, format="webp")
    os.remove(source)
    return str(destination)
