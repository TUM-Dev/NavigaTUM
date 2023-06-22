import json
import unittest
from pathlib import Path

from external.models import nat, roomfinder, tumonline, public_transport

RESULTS = Path(__file__).parent.parent / "results"


class NAT(unittest.TestCase):
    @staticmethod
    def test_buildings():
        """Test if the buildings can be loaded as nat.Building objects"""
        with open(RESULTS / "buildings_nat.json", encoding="utf-8") as file:
            for item in json.load(file):
                nat.Building(**item)

    @staticmethod
    def test_rooms():
        """Test if the rooms can be loaded as nat.Room objects"""
        with open(RESULTS / "rooms_nat.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                nat.Room(**item)

    @staticmethod
    def test_campus():
        """Test if the campi can be loaded as nat.Campus objects"""
        with open(RESULTS / "campus_nat.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                nat.Campus(**item)

    @staticmethod
    def test_org():
        """Test if the orgs can be loaded as nat.Organisation objects"""
        with open(RESULTS / "orgs_nat.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                nat.Organisation(**item)


class Roomfinder(unittest.TestCase):
    @staticmethod
    def test_maps():
        """Test if the maps can be loaded as roomfinder.Map objects"""
        with open(RESULTS / "maps_roomfinder.json", encoding="utf-8") as file:
            for item in json.load(file):
                roomfinder.Map(**item)

    @staticmethod
    def test_rooms():
        """Test if the rooms can be loaded as roomfinder.Room objects"""
        with open(RESULTS / "rooms_roomfinder.json", encoding="utf-8") as file:
            for item in json.load(file):
                roomfinder.Room(**item)

    @staticmethod
    def test_buildings():
        """Test if the buildings can be loaded as roomfinder.Building objects"""
        with open(RESULTS / "buildings_roomfinder.json", encoding="utf-8") as file:
            for item in json.load(file):
                roomfinder.Building(**item)


class TUMonline(unittest.TestCase):
    @staticmethod
    def test_rooms():
        """Test if the rooms can be loaded as tumonline.Room objects"""
        with open(RESULTS / "rooms_tumonline.json", encoding="utf-8") as file:
            for item in json.load(file):
                tumonline.Room(**item)

    @staticmethod
    def test_buildings():
        """Test if the buildings can be loaded as tumonline.Building objects"""
        with open(RESULTS / "buildings_tumonline.json", encoding="utf-8") as file:
            for item in json.load(file):
                tumonline.Building(**item)

    @staticmethod
    def test_orgs():
        """Test if the orgs can be loaded as tumonline.Organisation objects"""
        with open(RESULTS / "orgs-de_tumonline.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                tumonline.Organisation(**item)
        with open(RESULTS / "orgs-en_tumonline.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                tumonline.Organisation(**item)

class Public_Transport(unittest.TestCase):

    @staticmethod
    def test_stations():
        with open(RESULTS/"public_transport.json",encoding="utf-8") as file:
            for k,v in json.load(file).items():
                public_transport.Station(**v)


if __name__ == "__main__":
    unittest.main()
