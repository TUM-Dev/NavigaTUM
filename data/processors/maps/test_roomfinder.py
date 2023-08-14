import itertools
import typing
import unittest

from external.models import roomfinder
from processors.maps.models import Coordinate
from processors.maps.roomfinder import _calc_xy_of_coords_on_map, _merge_maps, _merge_str


class TestMerging(unittest.TestCase):
    def test_merge_identical(self):
        """Test if identical inputs are stripped of whitespace"""
        test_corpus = ["abc", " abc", "abc"]
        for s_1, s_2 in itertools.combinations(test_corpus, 2):
            self.assertEqual("abc", _merge_str(s_1, s_2))

    def test_merge_strings_happy_path(self):
        """Test if the merging of strings works as expected in the regular case"""
        self.assertEqual("Thierschbau 5/6. OG", _merge_str("Thierschbau 5. OG", "Thierschbau 6. OG"))
        self.assertEqual("Pre Something/Different Suf", _merge_str("Pre Something Suf", "Pre Different Suf"))
        self.assertEqual("Hello World/Universe", _merge_str("Hello World", "Hello Universe"))

    def test_merge_strings_subset(self):
        """Test if the merging of strings works as expected when one string is a subset of the other"""
        self.assertEqual("(Another) POSTFIX", _merge_str("POSTFIX", "Another POSTFIX"))
        self.assertEqual("(Another) POSTFIX", _merge_str("Another POSTFIX", "POSTFIX"))
        self.assertEqual("PREFIX (Another)", _merge_str("PREFIX", "PREFIX Another"))
        self.assertEqual("PREFIX (Another)", _merge_str("PREFIX Another", "PREFIX"))

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

        self.assertEqual(expected_map, _merge_maps(map1, map2))

    def test_merge_maps_unequal_keys(self):
        """Test if the merging of maps works as expected when the keyspace is not equal"""
        self.assertRaises(KeyError, _merge_maps, {"a": 1}, {"b": 2})  # different key in b
        self.assertRaises(KeyError, _merge_maps, {"a": 1}, {})  # no key in b
        self.assertEqual({}, _merge_maps({}, {"b": 2}))  # no key in a => empty map


class CoordinateToMap(unittest.TestCase):
    def test_coords_to_xy_center_rotation(self):
        """Test if xy coordinates are assigned correctly"""
        for rotation in range(360):
            with self.subTest(f"{rotation}° rotated around the center"):
                self.assertEqual(
                    (50, 50),
                    _calc_xy_of_coords_on_map(Coordinate(lat=0, lon=0), self.default_map(rotate=rotation)),
                )

    def test_coords_to_xy_translation(self):
        """Test if xy coordinates translate correctly"""
        for lon, lat in itertools.product(range(-100, 100), range(-100, 100)):
            actual_x, actual_y = _calc_xy_of_coords_on_map(Coordinate(lon=lon, lat=lat), self.default_map())
            self.assertAlmostEqual((lon + 100) / 200 * 100, actual_x, delta=0.6)
            self.assertAlmostEqual(100.0 - (lat + 100) / 200 * 100, actual_y, delta=0.6)

    def test_coords_to_xy_translation_rotation(self):
        """Test if xy coordinates translate and rotate correctly"""

        class Expected(typing.NamedTuple):
            coordinate: Coordinate
            expected: tuple[int, int]
            rotation: int

        expected = [
            Expected(Coordinate(lon=10, lat=10), (57, 50), 45),  # TODO: add more testcases
        ]
        for item in expected:
            self.assertEqual(
                item.expected,
                _calc_xy_of_coords_on_map(item.coordinate, self.default_map(rotate=item.rotation)),
            )

    @staticmethod
    def default_map(rotate=0) -> roomfinder.Map:
        """Create a basic map"""
        latlonbox = roomfinder.LatLonBox(
            west=-100,
            east=100,
            north=100,
            south=-100,
            rotation=rotate,
        )
        return roomfinder.Map(
            id="test-map",
            file="test-map.png",
            desc="Test Map",
            scale="10",
            latlonbox=latlonbox,
            width=100,
            height=100,
        )


if __name__ == "__main__":
    unittest.main()
