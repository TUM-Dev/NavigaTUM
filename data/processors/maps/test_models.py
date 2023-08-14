import unittest

from processors.maps import models


class ModelLoader(unittest.TestCase):
    def test_overlay(self) -> None:
        """Test if the overlay models can be loaded correctly"""
        with self.subTest(models.Overlay):
            models.Overlay.load_all()

    def test_custom(self) -> None:
        """Test if the custom map models can be loaded correctly"""
        with self.subTest("models.CustomBuildingMap without conversion"):
            models.CustomBuildingMap.load_all_raw()
        with self.subTest("models.CustomBuildingMap as roomfinder.Map"):
            models.CustomBuildingMap.load_all()


if __name__ == "__main__":
    unittest.main()
