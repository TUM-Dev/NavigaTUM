import json

import yaml


def merge_json(data, path):
    """
    Merge json data at path on top of the given data.
    This operates on the data dict directly without creating a copy.
    """
    with open(path, encoding="utf-8") as file:
        json_data = json.load(file)

    if not isinstance(json_data, dict):
        raise RuntimeError(f"Error: root node expected to be an object in file '{path}'")

    return _recursive_merge(data, json_data)


def merge_yaml(data, path):
    """
    Merge yaml data at path on top of the given data.
    This operates on the data dict directly without creating a copy.
    """
    with open(path, encoding="utf-8") as file:
        yaml_data = yaml.safe_load(file.read())

    if not isinstance(yaml_data, dict):
        raise RuntimeError(f"Error: root node expected to be an object in file '{path}'")

    # If the key of a root element is only numeric with 4 digits,
    # we assume it is a building id (which needs to be converted to string)
    ids_to_fix = []  # Cannot change dict while iterating
    for _id, _data in yaml_data.items():
        if isinstance(_id, int) and len(str(_id)) == 4:
            ids_to_fix.append(_id)
    for _id in ids_to_fix:
        yaml_data[str(_id)] = yaml_data[_id]
        del yaml_data[_id]

    return _recursive_merge(data, yaml_data)


def merge_object(data, obj, overwrite=True):
    """
    Merge the object on top of the given data.
    This operates on the data dict directly without creating a copy.
    The default behaviour is to overwrite the existing data.
    """
    return _recursive_merge(data, obj, overwrite)


def _recursive_merge(dict_a, dict_b, overwrite=True):
    """
    Recursively merge dict b on dict a (b overwrites a).
    Returns b if any of a or b is not a dict.
    This operates on dict a directly without creating a copy.
    """
    if not isinstance(dict_a, dict) or not isinstance(dict_b, dict):
        return dict_b if overwrite else dict_a

    for key, value in dict_b.items():
        if key in dict_a:
            dict_a[key] = _recursive_merge(dict_a[key], value, overwrite)
        else:
            dict_a[key] = value

    return dict_a
