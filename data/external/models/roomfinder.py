import json
import typing

from external.models.common import PydanticConfiguration, RESULTS_PATH


class RfMap(typing.NamedTuple):
    scale: str
    map_id: str
    name: str
    width: int
    height: int


# pylint: disable-next=too-many-instance-attributes
class Building(PydanticConfiguration):
    lat: float
    lon: float
    b_alias: str
    b_area: str
    b_id: str
    b_name: str
    default_map: RfMap | None
    maps: list[RfMap]
    b_room_count: int

    @classmethod
    def load_all(cls) -> list["Building"]:
        """Load all nat.Building's"""
        with (RESULTS_PATH / "buildings_roomfinder.json").open(encoding="utf-8") as file:
            return [cls.model_validate(item) for item in json.load(file)]


class LatLonBox(PydanticConfiguration):
    north: float
    south: float
    east: float
    west: float
    rotation: float


class Map(PydanticConfiguration):
    # pylint: disable-next=invalid-name
    id: str
    desc: str
    height: int
    width: int
    scale: str
    latlonbox: LatLonBox
    file: str
    source: str = "Roomfinder"

    @classmethod
    def load_all(cls) -> list["Map"]:
        """Load all nat.Map's"""
        with (RESULTS_PATH / "maps_roomfinder.json").open(encoding="utf-8") as file:
            return [cls(file=f"{item['id']}.webp", **item) for item in json.load(file)]


class RoomMetadata(PydanticConfiguration):
    m_desc: str
    m_name: str
    m_size: int
    m_type: str
    meta_id: int


class Room(PydanticConfiguration):
    # room specific properties
    lat: float
    lon: float
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
        with (RESULTS_PATH / "rooms_roomfinder.json").open(encoding="utf-8") as file:
            return [cls.model_validate(item) for item in json.load(file) if item]
