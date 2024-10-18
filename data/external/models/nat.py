import json

from external.models.common import PydanticConfiguration, RESULTS_PATH, TranslatableStr


class Building(PydanticConfiguration):
    building_code: str
    building_id: int | None
    building_name: str
    building_short: str | None
    address: str | None

    @classmethod
    def load_all(cls) -> list["Building"]:
        """Load all nat.Building's"""
        target = RESULTS_PATH / "buildings_nat.json"
        with target.open(encoding="utf-8") as file:
            return [cls.model_validate(item) for item in json.load(file)]


class Campus(PydanticConfiguration):
    campus: TranslatableStr
    campusshort: TranslatableStr

    @classmethod
    def load_all(cls) -> dict[str, "Campus"]:
        """Load all nat.Campus's"""
        target = RESULTS_PATH / "campus_nat.json"
        with target.open(encoding="utf-8") as file:
            return {key: cls.model_validate(item) for key, item in json.load(file).items()}


class Coordinate(PydanticConfiguration):
    lat: float | None
    lon: float | None
    source: str


class MapType(PydanticConfiguration):
    maptype_id: int
    maptype: TranslatableStr


class Map(PydanticConfiguration):
    map_id: int
    maptype: MapType
    url: str


class SeatingPlan(PydanticConfiguration):
    seat_count: int
    seating: TranslatableStr
    seating_id: int


# pylint: disable-next=too-many-instance-attributes
class Room(PydanticConfiguration):
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
    steckdosen: TranslatableStr | None = None
    org_id: int | None = None
    streaming_webcam: bool = False
    streaming_tumlive: bool = False

    @classmethod
    def load_all(cls) -> dict[str, "Room"]:
        """Load all nat.Room's"""
        target = RESULTS_PATH / "rooms_nat.json"
        with target.open(encoding="utf-8") as file:
            return {key: cls.model_validate(item) for key, item in json.load(file).items()}


class School(PydanticConfiguration):
    org_code: str
    org_name: TranslatableStr | str
    org_id: int
    org_url: str | None


class Organisation(PydanticConfiguration):
    org_code: str
    org_name: TranslatableStr
    org_type: str
    org_url: str | None
    school: School | None

    @classmethod
    def load_all(cls) -> dict[str, "Organisation"]:
        """Load all nat.Organisation's"""
        target = RESULTS_PATH / "orgs_nat.json"
        with target.open(encoding="utf-8") as file:
            return {key: cls.model_validate(item) for key, item in json.load(file).items()}
