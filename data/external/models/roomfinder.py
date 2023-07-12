import json
import typing

from external.models.common import PydanticConfiguration, RESULTS
from pydantic.dataclasses import dataclass


class RfMap(typing.NamedTuple):
    scale: str
    map_id: int
    name: str
    width: int
    height: int


@dataclass(config=PydanticConfiguration)
# pylint: disable-next=too-many-instance-attributes
class Building:
    b_alias: str
    b_area: str
    b_id: str
    b_name: str
    default_map: RfMap | None
    maps: list[RfMap]
    utm_easting: float
    utm_northing: float
    utm_zone: int
    b_room_count: int

    @classmethod
    def load_all(cls) -> list["Building"]:
        """Load all nat.Building's"""
        with open(RESULTS / "buildings_roomfinder.json", encoding="utf-8") as file:
            return [cls(**item) for item in json.load(file)]


@dataclass(config=PydanticConfiguration)
class LatLonBox:
    north: float
    south: float
    east: float
    west: float
    rotation: float


@dataclass(config=PydanticConfiguration)
class Map:
    # pylint: disable-next=invalid-name
    id: int
    desc: str | None
    height: int
    width: int
    scale: int
    latlonbox: LatLonBox | None = None

    @classmethod
    def load_all(cls) -> list["Map"]:
        """Load all nat.Map's"""
        with open(RESULTS / "maps_roomfinder.json", encoding="utf-8") as file:
            return [cls(**item) for item in json.load(file)]


@dataclass(config=PydanticConfiguration)
class RoomMetadata:
    m_desc: str
    m_name: str
    m_size: int
    m_type: str
    meta_id: int


@dataclass(config=PydanticConfiguration)
# pylint: disable-next=too-many-instance-attributes
class Room:
    # room specific properties
    utm_easting: float
    utm_northing: float
    utm_zone: int
    default_map: RfMap | None
    maps: list[RfMap]
    r_alias: str
    r_id: str
    r_level: str
    r_number: str
    metas: list[RoomMetadata]
    # building_properties
    b_alias: str
    b_area: str
    b_id: str
    b_name: str
    b_room_count: int = 0

    @classmethod
    def load_all(cls) -> list["Room"]:
        """Load all nat.Room's"""
        with open(RESULTS / "rooms_roomfinder.json", encoding="utf-8") as file:
            return [cls(**item) for item in json.load(file)]
