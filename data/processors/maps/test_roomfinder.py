import itertools
import typing
import unittest

from processors.maps.models import Coordinate
from processors.maps.roomfinder import _calc_xy_of_coords_on_map, _merge_maps, _merge_str


class TestMerging(unittest.TestCase):
    def test_merge_strings_happy_path(self):
        """Test if the merging of strings works as expected in the regular case"""
        self.assertEqual(_merge_str("Thierschbau 5. OG", "Thierschbau 6. OG"), "Thierschbau 5/6. OG")
        self.assertEqual(_merge_str("Pre Something Suf", "Pre Different Suf"), "Pre Something/Different Suf")
        self.assertEqual(_merge_str("Hello World", "Hello Universe"), "Hello World/Universe")
        self.assertEqual(_merge_str("Equal", "Equal"), "Equal")

    def test_merge_strings_subset(self):
        """Test if the merging of strings works as expected when one string is a subset of the other"""
        self.assertEqual(_merge_str("POSTFIX", "Another POSTFIX"), "(Another) POSTFIX")
        self.assertEqual(_merge_str("Another POSTFIX", "POSTFIX"), "(Another) POSTFIX")
        self.assertEqual(_merge_str("PREFIX", "PREFIX Another"), "PREFIX (Another)")
        self.assertEqual(_merge_str("PREFIX Another", "PREFIX"), "PREFIX (Another)")

    def test_merge_maps(self):
        """Test if the merging of maps works as expected"""
        map1 = {
            "id": 1,
            "name": "prefix John posfix",
            "age": 30,
            "favourite_float": 3.0,
            "location": {
                "city": "New York",
                "zip": 0,
            },
        }

        map2 = {
            "id": 2,
            "name": "prefix Tod posfix",
            "age": 25,
            "favourite_float": 5.0,
            "location": {
                "city": "San Francisco",
                "zip": 100,
            },
        }

        merged_map = _merge_maps(map1, map2)
        expected_map = {
            "id": 1,  # id should be taken from map1
            "name": "prefix John/Tod posfix",  # merged string
            "age": 27,  # average
            "favourite_float": 4.0,  # average
            "location": {
                "city": "New York/San Francisco",  # merged string
                "zip": 50,  # average
            },
        }

        self.assertEqual(merged_map, expected_map)

    def test_merge_maps_unequal_keys(self):
        """Test if the merging of maps works as expected when the keyspace is not equal"""
        self.assertRaises(KeyError, _merge_maps, {"a": 1}, {"b": 2})  # different key in b
        self.assertRaises(KeyError, _merge_maps, {"a": 1}, {})  # no key in b
        _merge_maps({}, {"b": 2})  # no key in a


class CoordinateToMap(unittest.TestCase):
    def test_coords_to_xy_center_rotation(self):
        """Test if xy coordinates are assigned correctly"""
        for rotation in range(360):
            with self.subTest(f"{rotation}° rotated around the center"):
                self.assertEqual(
                    _calc_xy_of_coords_on_map(Coordinate(lat=0, lon=0), self.default_map(rotate=rotation)),
                    (50, 50),
                )

    def test_coords_to_xy_translation(self):
        """Test if xy coordinates translate correctly"""
        for lon, lat in itertools.product(range(-100, 100), range(-100, 100)):
            actual_x, actual_y = _calc_xy_of_coords_on_map(Coordinate(lon=lon, lat=lat), self.default_map())
            self.assertAlmostEqual(actual_x, (lon + 100) / 200 * 100, delta=0.6)
            self.assertAlmostEqual(actual_y, 100.0 - (lat + 100) / 200 * 100, delta=0.6)

    def test_coords_to_xy_translation_rotation(self):
        """Test if xy coordinates translate and rotate correctly"""

        class Expected(typing.NamedTuple):
            coordinate: Coordinate
            expected: tuple[int, int]
            rotation: int

        expected = [
            Expected(Coordinate(lon=10, lat=10), (10, 10), 45),
        ]
        for item in expected:
            self.assertEqual(_calc_xy_of_coords_on_map(item.coordinate, self.default_map()), item.expected)

    @staticmethod
    def default_map(rotate=0):
        """Create a basic map"""
        return {
            "latlonbox": {
                "west": -100,
                "east": 100,
                "north": 100,
                "south": -100,
                "rotation": rotate,
            },
            "width": 100,
            "height": 100,
        }


if __name__ == "__main__":
    unittest.main()
