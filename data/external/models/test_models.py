import unittest

from external.models import nat, public_transport, roomfinder, tumonline


class ModelLoader(unittest.TestCase):
    def test_nat(self):
        """Test if the nat models can be loaded correctly"""
        with self.subTest(nat.Building):
            nat.Building.load_all()
        with self.subTest(nat.Room):
            nat.Room.load_all()
        with self.subTest(nat.Campus):
            nat.Campus.load_all()
        with self.subTest(nat.Organisation):
            nat.Organisation.load_all()

    def test_roomfinder(self):
        """Test if the roomfinder models can be loaded correctly"""
        with self.subTest(roomfinder.Map):
            roomfinder.Map.load_all()
        with self.subTest(roomfinder.Room):
            roomfinder.Room.load_all()
        with self.subTest(roomfinder.Building):
            roomfinder.Building.load_all()

    def test_tumonline(self):
        """Test if the tumonline models can be loaded correctly"""
        with self.subTest(tumonline.Room):
            tumonline.Room.load_all()
        with self.subTest(tumonline.Building):
            tumonline.Building.load_all()
        for lang in ("de", "en"):
            with self.subTest(tumonline.Organisation, lang=lang):
                tumonline.Organisation.load_all_for(lang)
        with self.subTest(tumonline.Usage):
            tumonline.Usage.load_all()

    def test_public_transport(self):
        """Test if the public_transport models can be loaded correctly"""
        with self.subTest(public_transport.Station):
            public_transport.Station.load_all()


if __name__ == "__main__":
    unittest.main()
