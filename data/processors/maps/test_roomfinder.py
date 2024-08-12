import itertools
import typing

import pytest
from external.models import roomfinder
from external.models.roomfinder import Coordinate
from processors.maps.roomfinder import _calc_xy_of_coords_on_map


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


@pytest.mark.parametrize("rotation", range(360))
def test_coords_to_xy_center_rotation(rotation: int) -> None:
    """Test if xy coordinates are assigned correctly"""
    print(f"{rotation}° rotated around the center")
    assign_map = default_map(rotate=rotation)
    assert _calc_xy_of_coords_on_map(
        Coordinate(lat=0, lon=0), assign_map.latlonbox, assign_map.width, assign_map.height
    ) == (50, 50)


@pytest.mark.parametrize("lat,lon", itertools.product(range(-10, 10), range(-10, 10)))
def test_coords_to_xy_translation(lon, lat) -> None:
    """Test if xy coordinates translate correctly"""
    assign_map = default_map()
    actual_x, actual_y = _calc_xy_of_coords_on_map(
        Coordinate(lon=lon, lat=lat), assign_map.latlonbox, assign_map.width, assign_map.height
    )
    expected_x = (lon + 100) / 200 * 100
    expected_y = 100.0 - (lat + 100) / 200 * 100
    assert expected_x - 0.6 < actual_x < expected_x + 0.6
    assert expected_y - 0.6 < actual_y < expected_y + 0.6


class ExpectedCoordinate(typing.NamedTuple):
    coordinate: Coordinate
    map: roomfinder.Map
    expected: tuple[int, int]


# purposely not parameterised for better performance
def test_coords_to_xy_off_map() -> None:
    """Test if xy coordinates translate correctly"""
    assign_map = default_map()
    for lat, lon in itertools.product(range(-200, 200), range(-200, 200)):
        actual = _calc_xy_of_coords_on_map(
            Coordinate(lon=lon, lat=lat),
            assign_map.latlonbox,
            assign_map.width,
            assign_map.height,
        )
        if (abs(lat) <= 100) and (abs(lon) <= 100):
            assert actual is not None
        else:
            assert actual is None


@pytest.mark.parametrize(
    "item",
    [
        ExpectedCoordinate(Coordinate(lon=10, lat=10), default_map(rotate=45), (57, 50)),  # TODO: add more testcases
    ],
)
def test_coords_to_xy_translation_rotation(item: ExpectedCoordinate) -> None:
    """Test if xy coordinates translate and rotate correctly"""
    assert (
        _calc_xy_of_coords_on_map(item.coordinate, item.map.latlonbox, item.map.width, item.map.height) == item.expected
    )
