import csv
from pathlib import Path
from typing import Any, TypeVar

import yaml
from processors.areatree.models import AreatreeBuidling
from utils import TranslatableStr

BASE_PATH = Path(__file__).parent.parent
SOURCES_PATH = BASE_PATH / "sources"
COORDINATES_CSV = SOURCES_PATH / "coordinates.csv"


def load_yaml(path: Path) -> Any:
    """
    Merge yaml data at path on top of the given data.

    This operates on the data dict directly without creating a copy.
    """

    def add_translatable_str(value: str | list[Any] | dict[str, Any]) -> Any:
        """Recursively change all {de: ..., en:...] to a TranslatableStr"""
        if isinstance(value, bool | float | int | str) or value is None:
            return value
        if isinstance(value, list):
            return [add_translatable_str(v) for v in value]
        if isinstance(value, dict):
            # We consider each dict that has only the keys "de" and "en" as translated string
            if set(value.keys()) == {"de", "en"}:
                return TranslatableStr(value["de"], value["en"])

            return {k: add_translatable_str(v) for k, v in value.items()}
        raise ValueError(f"Unhandled type {type(value)}")

    with path.open(encoding="utf-8") as file:
        yaml_data = yaml.safe_load(file.read())
    yaml_data = add_translatable_str(yaml_data)

    if not isinstance(yaml_data, dict):
        raise RuntimeError(f"Error: root node expected to be an object in file '{path}'")

    # If the key of a root element is only numeric with 4 digits,
    # we assume it is a building id (which needs to be converted to string)
    for _id, _data in list(yaml_data.items()):
        if isinstance(_id, int) and len(str(_id)) == 4:
            yaml_data[str(_id)] = yaml_data[_id]
            del yaml_data[_id]

    return yaml_data


def add_coordinates(data: dict) -> None:
    """
    Merge coordinates from CSV file on top of the given data.

    This operates on the data dict directly without creating a copy.
    """
    entries_which_should_not_exist = set()
    with COORDINATES_CSV.open("r", encoding="utf-8") as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            entry_id = row["id"]

            # Build coordinate object from CSV row
            coords = {"lat": float(row["lat"]), "lon": float(row["lon"])}

            # No additional columns needed - CSV contains only id, lat, lon
            if entry_id in data:
                recursively_merge(data, {entry_id: {"coords": coords}})
            else:
                entries_which_should_not_exist.add(entry_id)

    if entries_which_should_not_exist:
        raise RuntimeError(f"Coordinates exist for entries which should not exist: {entries_which_should_not_exist}")


A = TypeVar("A")
B = TypeVar("B")


def recursively_merge(dict_a: dict | A, dict_b: dict | B, overwrite: bool = True) -> dict | A | B:
    """
    Recursively merge dict b on dict a (b overwrites a).

    Returns b if any of a or b is not a dict.
    This operates on `dict_a` directly without creating a copy.
    """
    if not isinstance(dict_a, dict) or not isinstance(dict_b, dict):
        return dict_b if overwrite else dict_a

    for key, value in dict_b.items():
        if key in dict_a:
            dict_a[key] = recursively_merge(dict_a[key], value, overwrite)
        else:
            dict_a[key] = value

    return dict_a


def patch_areas(data: dict[str, AreatreeBuidling]) -> dict[str, dict[str, Any]]:
    """Merge areas from the yaml file at path on top of the given data."""
    areas_extended = SOURCES_PATH / "01_areas-extended.yaml"
    yaml_data = load_yaml(areas_extended)
    return recursively_merge(data, yaml_data)


def patch_rooms(data: dict[str, dict[str, Any]]) -> dict[str, dict[str, Any]]:
    """Merge rooms from the yaml file at path on top of the given data."""
    rooms_extended = SOURCES_PATH / "02_rooms-extended.yaml"
    yaml_data = load_yaml(rooms_extended)
    # make sure that the room id is in the name
    for key, value in yaml_data.items():
        if "name" in value and key not in value["name"]:
            value["name"] = f"{key} ({value['name']})"

    return recursively_merge(data, yaml_data)
