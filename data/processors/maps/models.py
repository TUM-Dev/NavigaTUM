import typing
from pathlib import Path

import yaml
from external.models.common import PydanticConfiguration
from pydantic.dataclasses import dataclass

BASE = Path(__file__).parent.parent.parent
EXTERNAL_RESULTS_PATH = BASE / "external" / "results"
SOURCES = BASE / "sources"


@dataclass(config=PydanticConfiguration)
class OverlayMap:
    file: str
    # pylint: disable-next=invalid-name
    id: int
    desc: str
    floor: str
    tumonline: str | None = None


@dataclass(config=PydanticConfiguration)
class OverlayProps:
    parent: str
    box: tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]]


@dataclass(config=PydanticConfiguration)
class Overlay:
    props: OverlayProps
    maps: list[OverlayMap]

    @classmethod
    def load_all(cls) -> dict[str, "Overlay"]:
        """Load all nat.Room's"""
        with open(SOURCES / "46_overlay-maps.yaml", encoding="utf-8") as file:
            return {_map["props"]["parent"]: cls(**_map) for _map in yaml.safe_load(file.read())}


class MapKey(typing.NamedTuple):
    building_id: str
    floor: str


class Coordinate(typing.TypedDict):
    lat: float
    lon: float
