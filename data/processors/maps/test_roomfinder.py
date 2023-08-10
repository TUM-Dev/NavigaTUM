import unittest

from processors.maps.roomfinder import _merge_maps, _merge_str


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


if __name__ == "__main__":
    unittest.main()
