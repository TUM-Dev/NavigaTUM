import typing

from external.models.common import PydanticConfiguration
from pydantic.dataclasses import dataclass

@dataclass(config=PydanticConfiguration)
class SubStation:
    id:str
    name:str
    lat:str
    lon:str
    parent:str

@dataclass(config=PydanticConfiguration)
class Station:
    id:str
    name:str
    lat:str
    lon:str
    sub_stations:list[SubStation]

