from external.models.common import PydanticConfiguration, TranslatableStr
from pydantic.dataclasses import dataclass


@dataclass(config=PydanticConfiguration)
class Building:
    building_code: str
    building_id: int | None
    building_name: str
    building_short: str | None
    address: str | None


@dataclass(config=PydanticConfiguration)
class Campus:
    campus: TranslatableStr
    campusshort: TranslatableStr


@dataclass(config=PydanticConfiguration)
class Coordinate:
    lat: float | None
    lon: float | None
    source: str


@dataclass(config=PydanticConfiguration)
class MapType:
    maptype_id: int
    maptype: TranslatableStr


@dataclass(config=PydanticConfiguration)
class Map:
    map_id: int
    maptype: MapType
    url: str


@dataclass(config=PydanticConfiguration)
class SeatingPlan:
    seat_count: int
    seating: TranslatableStr
    seating_id: int


@dataclass(config=PydanticConfiguration)
# pylint: disable-next=too-many-instance-attributes
class Room:
    description: str
    purpose: TranslatableStr
    purpose_id: int
    area: float
    coordinates: Coordinate
    eexam: TranslatableStr | None
    floor: str
    room_short: str | None
    maps: list[Map]
    schedule_url: str | None
    seatings: list[SeatingPlan]
    seats: int | None
    steckdosen: TranslatableStr | None
    streaming: str | None
    teaching: bool
    # semi-random sets of ids
    # pylint: disable-next=invalid-name
    id: str
    number: str
    ressource_id: int | None
    room_id: int | None
    room_identifier: str | None
    campus_id: str | None
    building_code: str
    org_id: int | None = None


@dataclass(config=PydanticConfiguration)
class Organisation:
    org_code: str
    org_name: TranslatableStr
    org_type: str
    org_url: str | None
