import json

import yaml


def merge_json(data, path):
    """
    Merge json data at path on top of the given data.
    This operates on the data dict directly without creating a copy.
    """
    with open(path) as f:
        json_data = json.load(f)

    if type(json_data) is not dict:
        raise RuntimeError(f"Error: root node expected to be an object in file '{path}'")

    data = _recursive_merge(data, json_data)
    return data


def merge_yaml(data, path):
    """
    Merge yaml data at path on top of the given data.
    This operates on the data dict directly without creating a copy.
    """
    with open(path) as f:
        yaml_data = yaml.safe_load(f.read())

    if type(yaml_data) is not dict:
        raise RuntimeError(f"Error: root node expected to be an object in file '{path}'")

    # If the key of a root element is only numeric with 4 digits,
    # we assume it is a building id (which needs to be converted to string)
    ids_to_fix = []  # Cannot change dict while iterating
    for _id, _data in yaml_data.items():
        if type(_id) is int and len(str(_id)) == 4:
            ids_to_fix.append(_id)
    for _id in ids_to_fix:
        yaml_data[str(_id)] = yaml_data[_id]
        del yaml_data[_id]

    data = _recursive_merge(data, yaml_data)
    return data


def merge_object(data, obj, overwrite=True):
    """
    Merge the object on top of the given data.
    This operates on the data dict directly without creating a copy.
    The default behaviour is to overwrite the existing data.
    """
    data = _recursive_merge(data, obj, overwrite)
    return data


def _recursive_merge(a, b, overwrite=True):
    """
    Recursively merge dict b on dict a (b overwrites a).
    Returns b if any of a or b is not a dict.
    This operates on dict a directly without creating a copy.
    """
    if type(a) is not dict or type(b) is not dict:
        return b if overwrite else a

    for k, v in b.items():
        if k in a:
            a[k] = _recursive_merge(a[k], v, overwrite)
        else:
            a[k] = v

    return a
