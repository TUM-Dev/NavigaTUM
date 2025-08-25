import json
import typing

import polars as pl
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
        df = pl.read_csv(
            RESULTS_PATH / "buildings_roomfinder.csv",
            dtypes={
                "b_id": pl.String,
                "b_alias": pl.String,
                "b_area": pl.String,
                "b_name": pl.String,
                "default_map_scale": pl.String,
                "default_map_id": pl.String,
                "default_map_name": pl.String,
            },
        )
        buildings = []
        for row in df.iter_rows(named=True):
            # Reconstruct default_map from flattened columns
            default_map = None
            if row["default_map_scale"] is not None:
                default_map = RfMap(
                    scale=str(row["default_map_scale"]),
                    map_id=str(row["default_map_id"]),
                    name=str(row["default_map_name"]),
                    width=int(row["default_map_width"]),
                    height=int(row["default_map_height"]),
                )

            # Reconstruct maps from JSON string
            maps_data = json.loads(row["maps_json"]) if row["maps_json"] else []
            maps = [
                RfMap(scale=str(m[0]), map_id=str(m[1]), name=str(m[2]), width=int(m[3]), height=int(m[4]))
                for m in maps_data
            ]

            building = cls(
                lat=float(row["lat"]),
                lon=float(row["lon"]),
                b_alias=str(row["b_alias"]) if row["b_alias"] else "",
                b_area=str(row["b_area"]) if row["b_area"] else "",
                b_id=str(row["b_id"]),
                b_name=str(row["b_name"]) if row["b_name"] else "",
                default_map=default_map,
                maps=maps,
                b_room_count=int(row["b_room_count"]),
            )
            buildings.append(building)
        return buildings


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
        df = pl.read_csv(
            RESULTS_PATH / "maps_roomfinder.csv",
            dtypes={
                "id": pl.String,
                "desc": pl.String,
                "scale": pl.String,
                "file": pl.String,
                "source": pl.String,
            },
        )
        maps = []
        for row in df.iter_rows(named=True):
            # Reconstruct latlonbox from flattened columns
            latlonbox = LatLonBox(
                north=float(row["latlonbox_north"]),
                south=float(row["latlonbox_south"]),
                east=float(row["latlonbox_east"]),
                west=float(row["latlonbox_west"]),
                rotation=float(row["latlonbox_rotation"]),
            )

            map_obj = cls(
                id=str(row["id"]),
                desc=str(row["desc"]) if row["desc"] else "",
                height=int(row["height"]),
                width=int(row["width"]),
                scale=str(row["scale"]) if row["scale"] else "",
                latlonbox=latlonbox,
                file=str(row["file"]),
                source=str(row["source"]),
            )
            maps.append(map_obj)
        return maps


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
        df = pl.read_csv(
            RESULTS_PATH / "rooms_roomfinder.csv",
            dtypes={
                "r_alias": pl.String,
                "r_id": pl.String,
                "r_level": pl.String,
                "r_number": pl.String,
                "b_alias": pl.String,
                "b_area": pl.String,
                "b_id": pl.String,
                "b_name": pl.String,
                "default_map_scale": pl.String,
                "default_map_id": pl.String,
                "default_map_name": pl.String,
            },
        )
        rooms = []
        for row in df.iter_rows(named=True):
            # Reconstruct default_map from flattened columns
            default_map = None
            if row["default_map_scale"] is not None:
                default_map = RfMap(
                    scale=str(row["default_map_scale"]),
                    map_id=str(row["default_map_id"]),
                    name=str(row["default_map_name"]),
                    width=int(row["default_map_width"]),
                    height=int(row["default_map_height"]),
                )

            # Reconstruct maps from JSON string
            maps_data = json.loads(row["maps_json"]) if row["maps_json"] else []
            maps = [
                RfMap(scale=str(m[0]), map_id=str(m[1]), name=str(m[2]), width=int(m[3]), height=int(m[4]))
                for m in maps_data
            ]

            # Reconstruct metas from JSON string
            metas_data = json.loads(row["metas_json"]) if row["metas_json"] else []
            metas = [
                RoomMetadata(
                    m_desc=str(m["m_desc"]) if m["m_desc"] else "",
                    m_name=str(m["m_name"]) if m["m_name"] else "",
                    m_size=int(m["m_size"]),
                    m_type=str(m["m_type"]) if m["m_type"] else "",
                    meta_id=int(m["meta_id"]),
                )
                for m in metas_data
            ]

            room = cls(
                lat=float(row["lat"]),
                lon=float(row["lon"]),
                default_map=default_map,
                maps=maps,
                r_alias=str(row["r_alias"]) if row["r_alias"] else "",
                r_id=str(row["r_id"]) if row["r_id"] else "",
                r_level=str(row["r_level"]) if row["r_level"] else "",
                r_number=str(row["r_number"]) if row["r_number"] else "",
                metas=metas,
                b_alias=str(row["b_alias"]) if row["b_alias"] else "",
                b_area=str(row["b_area"]) if row["b_area"] else "",
                b_id=str(row["b_id"]),
                b_name=str(row["b_name"]) if row["b_name"] else "",
                b_room_count=0,
            )
            rooms.append(room)
        return rooms
