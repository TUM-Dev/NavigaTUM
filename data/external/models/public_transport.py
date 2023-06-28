import typing

from external.models.common import PydanticConfiguration
from pydantic.dataclasses import dataclass

@dataclass(config=PydanticConfiguration)
class SubStation:
    id:str
    name:str
    lat:float
    lon:float
    parent:str

@dataclass(config=PydanticConfiguration)
class Station:
    id:str
    name:str
    lat:float
    lon:float
    sub_stations:list[SubStation]

