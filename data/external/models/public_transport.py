import json

from external.models.common import PydanticConfiguration, RESULTS
from pydantic.dataclasses import dataclass


@dataclass(config=PydanticConfiguration)
class SubStation:
    station_id: str
    name: str
    lat: float
    lon: float


@dataclass(config=PydanticConfiguration)
class Station:
    station_id: str
    name: str
    lat: float
    lon: float
    sub_stations: list[SubStation]

    @classmethod
    def load_all(cls) -> list["Station"]:
        """Load all public_transport.Station's"""
        with open(RESULTS / "public_transport.json", encoding="utf-8") as file:
            return [cls(**item) for item in json.load(file)]
