import itertools
import typing

import pytest
from external.models import roomfinder
from processors.maps.models import Coordinate
from processors.maps.roomfinder import _calc_xy_of_coords_on_map, _merge_maps, _merge_str


def default_map(rotate: int = 0) -> roomfinder.Map:
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


@pytest.mark.parametrize("s_1", ["abc", " abc", "abc"])
@pytest.mark.parametrize("s_2", ["abc", " abc", "abc"])
def test_merge_identical(s_1: str, s_2: str) -> None:
    """Test if identical inputs are stripped of whitespace"""
    assert _merge_str(s_1, s_2) == "abc"


def test_merge_strings_happy_path() -> None:
    """Test if the merging of strings works as expected in the regular case"""
    assert _merge_str("Thierschbau 5. OG", "Thierschbau 6. OG") == "Thierschbau 5/6. OG"
    assert _merge_str("Pre Something Suf", "Pre Different Suf") == "Pre Something/Different Suf"
    assert _merge_str("Hello World", "Hello Universe") == "Hello World/Universe"


def test_merge_strings_subset() -> None:
    """Test if the merging of strings works as expected when one string is a subset of the other"""
    assert _merge_str("POSTFIX", "Another POSTFIX") == "(Another) POSTFIX"
    assert _merge_str("Another POSTFIX", "POSTFIX") == "(Another) POSTFIX"
    assert _merge_str("PREFIX", "PREFIX Another") == "PREFIX (Another)"
    assert _merge_str("PREFIX Another", "PREFIX") == "PREFIX (Another)"


def test_merge_maps() -> None:
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

    assert expected_map == _merge_maps(map1, map2)


def test_merge_maps_unequal_keys() -> None:
    """Test if the merging of maps works as expected when the keyspace is not equal"""
    with pytest.raises(KeyError):
        assert _merge_maps({"a": 1}, {"b": 2})  # different key in b
    with pytest.raises(KeyError):
        _merge_maps({"a": 1}, {})  # no key in b
    assert _merge_maps({}, {"b": 2}) == {}  # no key in a => empty map


@pytest.mark.parametrize("rotation", range(360))
def test_coords_to_xy_center_rotation(rotation: int) -> None:
    """Test if xy coordinates are assigned correctly"""
    print(f"{rotation}° rotated around the center")
    assert _calc_xy_of_coords_on_map(Coordinate(lat=0, lon=0), default_map(rotate=rotation)) == (50, 50)


@pytest.mark.parametrize("lat,lon", itertools.product(range(-10, 10), range(-10, 10)))
def test_coords_to_xy_translation(lon, lat) -> None:
    """Test if xy coordinates translate correctly"""
    actual_x, actual_y = _calc_xy_of_coords_on_map(Coordinate(lon=lon, lat=lat), default_map())
    expected_x = (lon + 100) / 200 * 100
    expected_y = 100.0 - (lat + 100) / 200 * 100
    assert expected_x - 0.6 < actual_x < expected_x + 0.6
    assert expected_y - 0.6 < actual_y < expected_y + 0.6


class ExpectedCoordinate(typing.NamedTuple):
    coordinate: Coordinate
    map: roomfinder.Map
    expected: tuple[int, int]


@pytest.mark.parametrize(
    "item",
    [
        ExpectedCoordinate(Coordinate(lon=10, lat=10), default_map(rotate=45), (57, 50)),  # TODO: add more testcases
    ],
)
def test_coords_to_xy_translation_rotation(item: ExpectedCoordinate) -> None:
    """Test if xy coordinates translate and rotate correctly"""
    assert _calc_xy_of_coords_on_map(item.coordinate, item.map) == item.expected
