import unittest

from processors.maps import models


class ModelLoader(unittest.TestCase):
    def test_overlay(self):
        """Test if the overlay models can be loaded correctly"""
        with self.subTest(models.Overlay):
            models.Overlay.load_all()


if __name__ == "__main__":
    unittest.main()
