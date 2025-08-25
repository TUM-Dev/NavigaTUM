import json

import polars as pl
from external.models.common import PydanticConfiguration, RESULTS_PATH


class Address(PydanticConfiguration):
    place: str
    street: str
    zip_code: int


class Seats(PydanticConfiguration):
    sitting: int | None = None
    wheelchair: int | None = None
    standing: int | None = None


# pylint: disable-next=too-many-instance-attributes
class Room(PydanticConfiguration):
    address: Address
    seats: Seats
    floor_type: str
    floor_level: str
    tumonline_id: int
    area_id: int
    building_id: int
    main_operator_id: int
    usage_id: int
    alt_name: str | None = None
    arch_name: str | None = None
    calendar_resource_nr: int | None = None
    patched: bool = False

    @classmethod
    def load_all(cls) -> dict[str, "Room"]:
        """Load all tumonline.Room's"""
        df = pl.read_csv(
            RESULTS_PATH / "rooms_tumonline.csv",
            dtypes={
                "room_key": pl.String,
                "address_place": pl.String,
                "address_street": pl.String,
                "floor_type": pl.String,
                "floor_level": pl.String,
                "alt_name": pl.String,
                "arch_name": pl.String,
            },
        )
        rooms = {}
        for row in df.iter_rows(named=True):
            # Reconstruct address from flattened columns
            address = Address(
                place=str(row["address_place"]),
                street=str(row["address_street"]),
                zip_code=int(row["address_zip_code"]),
            )

            # Reconstruct seats from flattened columns
            seats = Seats(
                sitting=row["seats_sitting"] if row["seats_sitting"] is not None else None,
                wheelchair=row["seats_wheelchair"] if row["seats_wheelchair"] is not None else None,
                standing=row["seats_standing"] if row["seats_standing"] is not None else None,
            )

            room = cls(
                address=address,
                seats=seats,
                floor_type=str(row["floor_type"]),
                floor_level=str(row["floor_level"]),
                tumonline_id=int(row["tumonline_id"]),
                area_id=int(row["area_id"]),
                building_id=int(row["building_id"]),
                main_operator_id=int(row["main_operator_id"]),
                usage_id=int(row["usage_id"]),
                alt_name=str(row["alt_name"]) if row["alt_name"] else None,
                arch_name=str(row["arch_name"]) if row["arch_name"] else None,
                calendar_resource_nr=row["calendar_resource_nr"] if row["calendar_resource_nr"] is not None else None,
                patched=bool(row["patched"]),
            )
            rooms[str(row["room_key"])] = room
        return rooms


class Building(PydanticConfiguration):
    address: Address
    area_id: int
    name: str
    tumonline_id: int
    filter_id: int | None = None

    @classmethod
    def load_all(cls) -> dict[str, "Building"]:
        """Load all tumonline.Building's"""
        df = pl.read_csv(
            RESULTS_PATH / "buildings_tumonline.csv",
            dtypes={
                "building_key": pl.String,
                "address_place": pl.String,
                "address_street": pl.String,
                "name": pl.String,
            },
        )
        buildings = {}
        for row in df.iter_rows(named=True):
            # Reconstruct address from flattened columns
            address = Address(
                place=str(row["address_place"]),
                street=str(row["address_street"]),
                zip_code=int(row["address_zip_code"]),
            )

            building = cls(
                address=address,
                area_id=int(row["area_id"]),
                name=str(row["name"]),
                tumonline_id=int(row["tumonline_id"]),
                filter_id=row["filter_id"] if row["filter_id"] is not None else None,
            )
            buildings[str(row["building_key"])] = building
        return buildings


class Organisation(PydanticConfiguration):
    code: str
    name: str
    path: str

    @classmethod
    def load_all_for(cls, lang: str) -> dict[int, "Organisation"]:
        """Load all tumonline.Organisation's for a specific language"""
        df = pl.read_csv(
            RESULTS_PATH / f"orgs-{lang}_tumonline.csv",
            dtypes={
                "code": pl.String,
                "name": pl.String,
                "path": pl.String,
            },
        )
        organizations = {}
        for row in df.iter_rows(named=True):
            org = cls(code=str(row["code"]), name=str(row["name"]), path=str(row["path"]))
            organizations[int(row["org_id"])] = org
        return organizations


class Usage(PydanticConfiguration):
    din277_id: str
    din277_name: str
    name: str

    @classmethod
    def load_all(cls) -> dict[int, "Usage"]:
        """Load all tumonline.Usage's"""
        df = pl.read_csv(
            RESULTS_PATH / "usages_tumonline.csv",
            dtypes={
                "din277_id": pl.String,
                "din277_name": pl.String,
                "name": pl.String,
            },
        )
        usages = {}
        for row in df.iter_rows(named=True):
            usage = cls(din277_id=str(row["din277_id"]), din277_name=str(row["din277_name"]), name=str(row["name"]))
            usages[int(row["usage_id"])] = usage
        return usages
