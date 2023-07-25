from pathlib import Path
from typing import Any, TypeVar, Union

import yaml
from processors.areatree.models import AreatreeBuidling
from utils import TranslatableStr


def load_yaml(path: str) -> Any:
    """
    Merge yaml data at path on top of the given data.
    This operates on the data dict directly without creating a copy.
    """

    def add_translatable_str(value: Union[str, list[Any], dict[str, Any]]) -> Any:
        """Recursively change all {de: ..., en:...] to a TranslatableStr"""
        if isinstance(value, (bool, float, int, str)) or value is None:
            return value
        if isinstance(value, list):
            return [add_translatable_str(v) for v in value]
        if isinstance(value, dict):
            # We consider each dict that has only the keys "de" and "en" as translated string
            if set(value.keys()) == {"de", "en"}:
                return TranslatableStr(value["de"], value["en"])

            return {k: add_translatable_str(v) for k, v in value.items()}
        raise ValueError(f"Unhandled type {type(value)}")

    with open(path, encoding="utf-8") as file:
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


def add_coordinates(data: dict, path: str) -> None:
    """
    Merge coordinates from yaml files placed at path on top of the given data.
    (Merging happens in alphanumeric order, so later files would overwrite earlier files)
    This operates on the data dict directly without creating a copy.
    """
    for file in sorted(Path(path).iterdir()):
        yaml_data = load_yaml(file)

        recursively_merge(data, {_id: {"coords": val} for _id, val in yaml_data.items()})


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


def patch_areas(data: dict[str, AreatreeBuidling], path: str) -> dict[str, dict[str, Any]]:
    """
    Merge areas from the yaml file at path on top of the given data.
    """
    yaml_data = load_yaml(path)
    return recursively_merge(data, yaml_data)


def patch_rooms(data: dict[str, dict[str, Any]], path: str) -> dict[str, dict[str, Any]]:
    """
    Merge rooms from the yaml file at path on top of the given data.
    """
    yaml_data = load_yaml(path)
    # make sure that the room id is in the name
    for key, value in yaml_data.items():
        if "name" in value and key not in value["name"]:
            value["name"] = f"{key} ({value['name']})"

    return recursively_merge(data, yaml_data)
