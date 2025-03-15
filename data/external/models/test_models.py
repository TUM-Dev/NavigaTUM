from typing import Literal

import pytest

from external.models import public_transport, roomfinder, tumonline


def test_roomfinder_map():
    """Load all maps from the roomfinder.Map"""
    roomfinder.Map.load_all()


def test_roomfinder_room():
    """Load all rooms from the roomfinder.Room"""
    roomfinder.Room.load_all()


def test_roomfinder_building():
    """Load all buildings from the roomfinder.Building"""
    roomfinder.Building.load_all()


def test_tumonline_room():
    """Load all rooms from the tumonline.Room"""
    tumonline.Room.load_all()


def test_tumonline_building():
    """Load all buildings from the tumonline.Building"""
    tumonline.Building.load_all()


@pytest.mark.parametrize("lang", ["de", "en"])
def test_tumonline_org(lang: Literal["de", "en"]):
    """Load all orgs from the tumonline.Organisation"""
    tumonline.Organisation.load_all_for(lang)


def test_tumonline_usage():
    """Load all usages from the tumonline.Usage"""
    tumonline.Usage.load_all()


def test_public_transport():
    """Load all stations from the public_transport.Station"""
    public_transport.Station.load_all()
