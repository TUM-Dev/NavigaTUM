from processors.maps import models


def test_overlay() -> None:
    """Test if the overlay models can be loaded correctly"""
    models.Overlay.load_all()


def test_custom_without_conversion() -> None:
    """Test if the custom map models can be loaded correctly without conversion"""
    models.CustomBuildingMap.load_all_raw()


def test_custom_as_roomfinder_map() -> None:
    """Test if the custom map models can be loaded correctly as roomfinder.Map"""
    models.CustomBuildingMap.load_all()
