import json

from external.models.common import PydanticConfiguration, RESULTS, TranslatableStr
from pydantic.dataclasses import dataclass


@dataclass(config=PydanticConfiguration)
class Building:
    building_code: str
    building_id: int | None
    building_name: str
    building_short: str | None
    address: str | None

    @classmethod
    def load_all(cls) -> list["Building"]:
        """Load all nat.Building's"""
        with open(RESULTS / "buildings_nat.json", encoding="utf-8") as file:
            return [cls(**item) for item in json.load(file)]


@dataclass(config=PydanticConfiguration)
class Campus:
    campus: TranslatableStr
    campusshort: TranslatableStr

    @classmethod
    def load_all(cls) -> dict[str, "Campus"]:
        """Load all nat.Campus's"""
        with open(RESULTS / "campus_nat.json", encoding="utf-8") as file:
            return {key: cls(**item) for key, item in json.load(file).items()}


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

    @classmethod
    def load_all(cls) -> dict[str, "Room"]:
        """Load all nat.Room's"""
        with open(RESULTS / "rooms_nat.json", encoding="utf-8") as file:
            return {key: cls(**item) for key, item in json.load(file).items()}


@dataclass(config=PydanticConfiguration)
class School:
    org_code: str
    org_name: TranslatableStr | str
    org_id: int


@dataclass(config=PydanticConfiguration)
class Organisation:
    org_code: str
    org_name: TranslatableStr
    org_type: str
    org_url: str | None
    school: School | None

    @classmethod
    def load_all(cls) -> dict[str, "Organisation"]:
        """Load all nat.Organisation's"""
        with open(RESULTS / "orgs_nat.json", encoding="utf-8") as file:
            return {key: cls(**item) for key, item in json.load(file).items()}
