import logging
from typing import TypedDict

# python 3.11 feature => move to typing when 3.11 is mainstream
from typing_extensions import NotRequired


def read_areatree():
    """Reads the areatree file and the basic data, gained from the areatree"""

    parent_stack: list[str] = []
    last_element: str = ""
    data = {}
    for line in _areatree_lines():
        indent = len(line) - len(line.lstrip(" "))
        if indent % 2 != 0:
            raise RuntimeError(f"Indentation not multiple of 2: '{line}'")
        if (indent // 2) > len(parent_stack):
            parent_stack.append(last_element)
        elif (indent // 2) < len(parent_stack):
            parent_stack = parent_stack[: indent // 2]

        building_data = _parse_areatree_line(line)
        last_element = building_data["id"]
        data[building_data["id"]] = {"parents": parent_stack[:], **building_data}
    return data


def _areatree_lines():
    """
    Generator that yields lines from the areatree file

    ignores:
    - Empty lines,
    - comment lines and
    - comments in lines
    """

    with open("sources/00_areatree", encoding="utf-8") as file:
        for line in file:
            # Empty lines and comment lines are ignored
            line = line.split("#")[0]
            if not line.strip():
                continue
            yield line


def _split_line(line: str) -> tuple[str, str, str]:
    """
    Splits a line from the areatree file into the three parts
    The syntax is building-id(s):name(s):internal-id[,visible_id]
    """
    parts = line.split(":")
    if len(parts) != 3:
        raise RuntimeError(f"Invalid line, expected 3 ':'-separated parts: '{line}'")
    internal_id: str
    raw_names: str
    building_ids: str
    (building_ids, raw_names, internal_id) = parts
    return building_ids.strip(), raw_names.strip(), internal_id.strip()


class BuildingPrefix(TypedDict):
    data_quality: NotRequired[dict[str, bool]]
    b_prefix: NotRequired[str | list[str]]


class IdType(TypedDict):
    id: str
    visible_id: NotRequired[str]
    type: str


class Names(TypedDict):
    name: str
    short_name: NotRequired[str]


class AreatreeBuidling(TypedDict):
    data_quality: NotRequired[dict[str, bool]]
    b_prefix: NotRequired[str | list[str]]
    id: str
    visible_id: NotRequired[str]
    type: str
    name: str
    short_name: NotRequired[str]


def _parse_areatree_line(line: str) -> AreatreeBuidling:
    """Parses a line from the areatree file to reveal the correct parent and children"""
    (building_ids, raw_names, internal_id) = _split_line(line)

    building_data = _extract_building_prefix(building_ids)
    names = _extract_names(raw_names.split("|"))
    id_and_type = _extract_id_and_type(internal_id, building_data.get("b_prefix"))
    # we merge the results like this for mypy to be happy, sigh
    result: AreatreeBuidling = {
        "id": id_and_type["id"],
        "type": id_and_type["type"],
        "name": names["name"],
    }
    if "data_quality" in building_data:
        result["data_quality"] = building_data["data_quality"]
    if "b_prefix" in building_data:
        result["b_prefix"] = building_data["b_prefix"]
    if "visible_id" in id_and_type:
        result["visible_id"] = id_and_type["visible_id"]
    if "short_name" in names:
        result["short_name"] = names["short_name"]
    return result


def _extract_id_and_type(internal_id: str, b_prefix: str | list[str] | None) -> IdType:
    """Extracts the id and type from the internal_id"""
    results: IdType = {"id": "", "type": ""}
    if "[" in internal_id:
        internal_id, results["type"] = internal_id.removesuffix("]").split("[")
    if "," in internal_id:
        ids = internal_id.split(",")
        if len(ids) != 2:
            raise RuntimeError(f"More than two ids found: '{internal_id}'")
        results["id"], results["visible_id"] = ids
    elif internal_id:
        results["id"] = internal_id
    elif isinstance(b_prefix, str) and b_prefix:
        results["id"] = b_prefix
    else:
        raise RuntimeError(f"No id provided in line: '{internal_id}'")
    # we infer which type some elements are, if they have not specified it
    if not results["type"]:
        results["type"] = "building" if results["id"] == b_prefix else "area"
    return results


def _extract_building_prefix(building_ids: str) -> BuildingPrefix:
    """Extracts the building prefix from the building_ids"""
    results: BuildingPrefix = {}
    # areatree_uncertain
    if "-" in building_ids:
        results["data_quality"] = {"areatree_uncertain": True}
        building_ids = building_ids.replace("-", "")

    # b_prefix
    if "," in building_ids:
        results["b_prefix"] = building_ids.split(",")
    elif building_ids:
        results["b_prefix"] = building_ids
    return results


def _extract_names(names: list[str]) -> Names:
    """Extracts the name and the possible short_name"""
    building_data: Names = {"name": names[0]}
    if len(names) == 2:
        if len(names[1]) > 20:
            logging.warning(f"'{names[1]}' is very long for a short name (>20 chars)")

        building_data["short_name"] = names[1]
    elif len(names) > 2:
        raise RuntimeError(f"Too many names: {names}")
    return building_data
