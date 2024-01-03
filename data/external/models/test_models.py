from typing import Literal

import pytest
from external.models import nat, public_transport, roomfinder, tumonline


def test_nat_building():
    nat.Building.load_all()


def test_nat_room():
    nat.Room.load_all()


def test_nat_campus():
    nat.Campus.load_all()


def test_nat_org():
    nat.Organisation.load_all()


def test_roomfinder_map():
    """Test if the roomfinder.Map model can be loaded correctly"""
    roomfinder.Map.load_all()


def test_roomfinder_room():
    """Test if the roomfinder.Room model can be loaded correctly"""
    roomfinder.Room.load_all()


def test_roomfinder_building():
    roomfinder.Building.load_all()


def test_tumonline_room():
    tumonline.Room.load_all()


def test_tumonline_room():
    tumonline.Building.load_all()


@pytest.mark.parametrize("lang", ["de", "en"])
def test_tumonline_room(lang: Literal["de", "en"]):
    tumonline.Organisation.load_all_for(lang)


def test_tumonline_usage():
    tumonline.Usage.load_all()


def test_public_transport():
    public_transport.Station.load_all()
