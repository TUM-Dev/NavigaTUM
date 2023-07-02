import json
import unittest
from pathlib import Path

from external.models import nat, public_transport, roomfinder, tumonline

RESULTS = Path(__file__).parent.parent / "results"


class NAT(unittest.TestCase):
    def test_buildings(self):
        """Test if the buildings can be loaded as nat.Building objects"""
        with open(RESULTS / "buildings_nat.json", encoding="utf-8") as file:
            for item in json.load(file):
                with self.subTest(item=item):
                    nat.Building(**item)

    def test_rooms(self):
        """Test if the rooms can be loaded as nat.Room objects"""
        with open(RESULTS / "rooms_nat.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                with self.subTest(item=item):
                    nat.Room(**item)

    def test_campus(self):
        """Test if the campi can be loaded as nat.Campus objects"""
        with open(RESULTS / "campus_nat.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                with self.subTest(item=item):
                    nat.Campus(**item)

    def test_org(self):
        """Test if the orgs can be loaded as nat.Organisation objects"""
        with open(RESULTS / "orgs_nat.json", encoding="utf-8") as file:
            for item in json.load(file).values():
                with self.subTest(item=item):
                    nat.Organisation(**item)


class Roomfinder(unittest.TestCase):
    def test_maps(self):
        """Test if the maps can be loaded as roomfinder.Map objects"""
        with open(RESULTS / "maps_roomfinder.json", encoding="utf-8") as file:
            for item in json.load(file):
                with self.subTest(item=item):
                    roomfinder.Map(**item)

    def test_rooms(self):
        """Test if the rooms can be loaded as roomfinder.Room objects"""
        with open(RESULTS / "rooms_roomfinder.json", encoding="utf-8") as file:
            for item in json.load(file):
                with self.subTest(item=item):
                    roomfinder.Room(**item)

    def test_buildings(self):
        """Test if the buildings can be loaded as roomfinder.Building objects"""
        with open(RESULTS / "buildings_roomfinder.json", encoding="utf-8") as file:
            for item in json.load(file):
                with self.subTest(item=item):
                    roomfinder.Building(**item)


class TUMonline(unittest.TestCase):
    def test_rooms(self):
        """Test if the rooms can be loaded as tumonline.Room objects"""
        with open(RESULTS / "rooms_tumonline.json", encoding="utf-8") as file:
            for item in json.load(file):
                with self.subTest(item=item):
                    tumonline.Room(**item)

    def test_buildings(self):
        """Test if the buildings can be loaded as tumonline.Building objects"""
        with open(RESULTS / "buildings_tumonline.json", encoding="utf-8") as file:
            for item in json.load(file):
                with self.subTest(item=item):
                    tumonline.Building(**item)

    def test_orgs(self):
        """Test if the orgs can be loaded as tumonline.Organisation objects"""
        for lang in ("de", "en"):
            with open(RESULTS / f"orgs-{lang}_tumonline.json", encoding="utf-8") as file:
                for item in json.load(file).values():
                    with self.subTest(item=item, lang=lang):
                        tumonline.Organisation(**item)


class PublicTransport(unittest.TestCase):
    def test_stations(self):
        """Test if the stations can be loaded as public_transport.Station objects"""
        with open(RESULTS / "public_transport.json", encoding="utf-8") as file:
            for item in json.load(file):
                with self.subTest(item=item):
                    public_transport.Station(**item)


if __name__ == "__main__":
    unittest.main()
