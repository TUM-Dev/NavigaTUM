import json

from external.models.common import PydanticConfiguration, RESULTS


class Address(PydanticConfiguration):
    floor: str
    place: str
    street: str
    zip_code: str


class Seats(PydanticConfiguration):
    sitting: int | None = None
    wheelchair: int | None = None
    standing: int | None = None


# pylint: disable-next=too-many-instance-attributes
class Room(PydanticConfiguration):
    address: Address
    seats: Seats
    area_id: int
    building_id: int
    floor_type: str
    main_operator_id: int
    usage_id: int
    arch_name: str | None = None
    calendar_resource_nr: int | None = None
    patched: bool = False

    @classmethod
    def load_all(cls) -> dict[str, "Room"]:
        """Load all tumonline.Room's"""
        with open(RESULTS / "rooms_tumonline.json", encoding="utf-8") as file:
            return {key: cls.model_validate(item) for key, item in json.load(file).items()}


class Building(PydanticConfiguration):
    area_id: int
    filter_id: int
    name: str

    @classmethod
    def load_all(cls) -> list["Building"]:
        """Load all tumonline.Building's"""
        with open(RESULTS / "buildings_tumonline.json", encoding="utf-8") as file:
            return [cls.model_validate(item) for item in json.load(file)]


class Organisation(PydanticConfiguration):
    # pylint: disable-next=invalid-name
    id: int
    code: str
    name: str
    path: str

    @classmethod
    def load_all_for(cls, lang: str) -> dict[int, "Organisation"]:
        """Load all tumonline.Organisation's for a specific language"""
        with open(RESULTS / f"orgs-{lang}_tumonline.json", encoding="utf-8") as file:
            return {key: cls.model_validate(item) for key, item in json.load(file).items()}


class Usage(PydanticConfiguration):
    # pylint: disable-next=invalid-name
    id: int
    din277_id: str
    din277_name: str
    name: str

    @classmethod
    def load_all(cls) -> dict[int, "Usage"]:
        """Load all tumonline.Usage's"""
        with open(RESULTS / "usages_tumonline.json", encoding="utf-8") as file:
            return {item["id"]: cls.model_validate(item) for item in json.load(file)}
