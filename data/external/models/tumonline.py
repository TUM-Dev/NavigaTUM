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
            schema_overrides={
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




