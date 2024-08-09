import hashlib

from processors.maps import models
from processors.maps.roomfinder import SITE_PLANS_PATH


def test_overlay() -> None:
    """Test if the overlay models can be loaded correctly"""
    models.Overlay.load_all()


def test_custom_without_conversion() -> None:
    """Test if the custom map models can be loaded correctly without conversion"""
    models.CustomBuildingMap.load_all_raw()


def test_custom_as_roomfinder_map() -> None:
    """Test if the custom map models can be loaded correctly as roomfinder.Map"""
    models.CustomBuildingMap.load_all()


def test_no_duplicate_plans() -> None:
    """Remove content 1:1 duplicates from the maps_list"""
    content_to_filename_dict: dict[str, str] = {}
    for filename in SITE_PLANS_PATH.glob("*.webp"):
        file_hash = hashlib.sha256(filename.read_bytes(), usedforsecurity=False).hexdigest()
        _id = filename.with_suffix("").name
        assert file_hash not in content_to_filename_dict
        content_to_filename_dict[file_hash] = _id
