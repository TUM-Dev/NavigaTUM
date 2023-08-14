import json

from external.models.common import PydanticConfiguration, RESULTS


class SubStation(PydanticConfiguration):
    station_id: str
    name: str
    lat: float
    lon: float


class Station(PydanticConfiguration):
    station_id: str
    name: str
    lat: float
    lon: float
    sub_stations: list[SubStation]

    @classmethod
    def load_all(cls) -> list["Station"]:
        """Load all public_transport.Station's"""
        with open(RESULTS / "public_transport.json", encoding="utf-8") as file:
            return [cls.model_validate(item) for item in json.load(file)]
