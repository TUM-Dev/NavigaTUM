import os
from pathlib import Path

from PIL import Image


def convert_to_webp(source: Path):
    """Convert image to WebP.

    Args:
        source (pathlib.Path): Path to source image

    Returns:
        pathlib.Path: path to new image
    """
    destination = Path(source).with_suffix(".webp")

    image = Image.open(source)
    image.save(destination, format="webp")
    os.remove(source)
    return str(destination)
