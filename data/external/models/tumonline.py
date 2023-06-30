from external.models.common import PydanticConfiguration
from pydantic.dataclasses import dataclass


@dataclass(config=PydanticConfiguration)
# pylint: disable-next=too-many-instance-attributes
class ExtendedRoomData:
    address: str
    building: str
    zip_code_location: str
    room_number: str
    floor_number: str
    floor_type: str
    area_m2: float
    architect_room_nr: str
    additional_description: str
    purpose: str
    wheelchair_spaces: int
    standing_places: int
    seats: int


@dataclass(config=PydanticConfiguration)
# pylint: disable-next=too-many-instance-attributes
class Room:
    address: str
    address_link: str
    alt_name: str
    arch_name: str
    b_area_id: int
    b_filter_id: int
    calendar: str | None
    list_index: str
    op_link: str
    operator: str
    plz_place: str
    room_link: str
    roomcode: str
    usage: int
    extended: ExtendedRoomData | None = None


@dataclass(config=PydanticConfiguration)
class Building:
    area_id: int
    filter_id: int
    name: str


@dataclass(config=PydanticConfiguration)
class Organisation:
    # pylint: disable-next=invalid-name
    id: int
    code: str
    name: str
    path: str
