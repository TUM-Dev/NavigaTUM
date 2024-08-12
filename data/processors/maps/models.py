import typing
from pathlib import Path

import yaml
from external.models import roomfinder
from external.models.common import PydanticConfiguration
from PIL import Image

BASE_PATH = Path(__file__).parent.parent.parent
SITE_PLANS_PATH = BASE_PATH / "sources" / "img" / "maps" / "site_plans"
SOURCES_PATH = BASE_PATH / "sources"


class OverlayMap(PydanticConfiguration):
    file: str
    floor_index: int
    desc: str
    floor: str
    tumonline: str | None = None


class OverlayProps(PydanticConfiguration):
    parent: str
    box: tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]]


class Overlay(PydanticConfiguration):
    props: OverlayProps
    maps: list[OverlayMap]

    @classmethod
    def load_all(cls) -> dict[str, "Overlay"]:
        """Load all nat.Room's"""
        with (SOURCES_PATH / "46_overlay-maps.yaml").open(encoding="utf-8") as file:
            return {_map["props"]["parent"]: cls.model_validate(_map) for _map in yaml.safe_load(file.read())}


class MapKey(typing.NamedTuple):
    building_id: str
    floor: str


class CustomMapProps(PydanticConfiguration):
    scale: str
    north: float
    east: float
    south: float
    west: float
    rotation: float
    source: str = "NavigaTUM-Contributors"


class ImageDimensions(typing.TypedDict):
    width: int
    height: int


class CustomMapItem(PydanticConfiguration):
    file: str
    b_id: str
    desc: str
    floor: str

    def dimensions(self) -> ImageDimensions:
        """Get the dimensions of the image"""
        with Image.open(SITE_PLANS_PATH / self.file) as img:
            return {"width": img.width, "height": img.height}


class CustomBuildingMap(PydanticConfiguration):
    props: CustomMapProps
    maps: list[CustomMapItem]

    @classmethod
    def load_all_raw(cls) -> list["CustomBuildingMap"]:
        """Load all nat.Room's"""
        with (SOURCES_PATH / "45_custom-maps.yaml").open(encoding="utf-8") as file:
            return [cls.model_validate(_map) for _map in yaml.safe_load(file.read())]

    def as_roomfinder_maps(self) -> dict[MapKey, roomfinder.Map]:
        """Convert to roomfinder.Map"""
        return {
            MapKey(_map.b_id, _map.floor): roomfinder.Map(
                desc=_map.desc,
                id=".".join(_map.file.split(".")[:-1]),
                file=_map.file,
                source=self.props.source,
                scale=self.props.scale,
                latlonbox=roomfinder.LatLonBox(
                    north=self.props.north,
                    east=self.props.east,
                    west=self.props.west,
                    south=self.props.south,
                    rotation=self.props.rotation,
                ),
                **_map.dimensions(),
            )
            for _map in self.maps
        }

    @classmethod
    def load_all(cls) -> dict[MapKey, roomfinder.Map]:
        """Load all custom maps as roomfinder.Map's"""
        results: dict[MapKey, roomfinder.Map] = {}
        for _map in cls.load_all_raw():
            results |= _map.as_roomfinder_maps()
        return results
