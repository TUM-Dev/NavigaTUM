from external.models.common import PydanticConfiguration
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
