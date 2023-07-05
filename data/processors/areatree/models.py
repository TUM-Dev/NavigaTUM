from typing import TypedDict

# python 3.11 feature => move to typing when 3.11 is mainstream
from typing_extensions import NotRequired


class BuildingPrefix(TypedDict):
    data_quality: NotRequired[dict[str, bool]]
    b_prefix: NotRequired[str | list[str]]


class IdType(TypedDict):
    id: str
    visible_id: NotRequired[str | list[str]]
    type: str


class Names(TypedDict):
    name: str
    short_name: NotRequired[str]


class AreatreeBuidling(TypedDict):
    data_quality: NotRequired[dict[str, bool]]
    b_prefix: NotRequired[str | list[str]]
    id: str
    visible_id: NotRequired[str | list[str]]
    type: str
    name: str
    short_name: NotRequired[str]
