import logging
from pathlib import Path
from collections.abc import Iterator

from processors.areatree import models

AREATREE_FILE = Path(__file__).parent / "config.areatree"


def read_areatree() -> dict[str, models.AreatreeBuidling]:
    """Read the areatree file and the basic data, gained from the areatree"""
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

        building_data = _parse_areatree_line(line, parent_stack[:])
        data[building_data["id"]] = building_data
        last_element = building_data["id"]
    return data


def _areatree_lines() -> Iterator[str]:
    """
    Extract the lines from the areatree file via a generator pattern

    Ignores:
    - Empty lines,
    - comment lines and
    - comments in lines
    """
    with AREATREE_FILE.open(encoding="utf-8") as file:
        for line in file:
            line_without_comments = line.split("#")[0]
            if line_without_comments_and_whitespace := line_without_comments.rstrip():
                yield line_without_comments_and_whitespace


def _split_line(line: str) -> tuple[str, str, str]:
    """
    Split a line from the areatree file into the three parts (building-ids,name,internal-id)

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


def _parse_areatree_line(line: str, parents: list[str]) -> models.AreatreeBuidling:
    """Parse a line from the areatree file to reveal the correct parent and children"""
    (building_ids, raw_names, internal_id) = _split_line(line)

    building_data = _extract_building_prefix(building_ids)
    names = _extract_names(raw_names.split("|"))
    id_and_type = _extract_id_and_type(internal_id, building_data.get("b_prefix"))
    # we merge the results like this for mypy to be happy, sigh
    result: models.AreatreeBuidling = {
        "id": id_and_type["id"],
        "type": id_and_type["type"],
        "name": names["name"],
        "parents": parents,
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


def _extract_id_and_type(internal_id: str, b_prefix: str | list[str] | None) -> models.IdType:
    """Extract the id and type from the internal_id"""
    results: models.IdType = {"id": "", "type": ""}
    if "[" in internal_id:
        internal_id, results["type"] = internal_id.removesuffix("]").split("[")
        if "," in results["type"]:
            raise RuntimeError(
                f"can't parse {internal_id}."
                f"The type has to be specified after specifying visible_ids."
                f"try :id,visible_ids[type] instead",
            )
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


def _extract_building_prefix(building_ids: str) -> models.BuildingPrefix:
    """Extract the building prefix from the building_ids"""
    results: models.BuildingPrefix = {}
    # areatree_uncertain
    if building_ids.startswith("-"):
        results["data_quality"] = {"areatree_uncertain": True}
        building_ids = building_ids.lstrip("-")

    # b_prefix
    if "," in building_ids:
        results["b_prefix"] = building_ids.split(",")
    elif building_ids:
        results["b_prefix"] = building_ids
    return results


def _extract_names(names: list[str]) -> models.Names:
    """Extract the name and the possible short_name"""
    building_data: models.Names = {"name": names[0]}
    if len(names) == 2:
        if len(names[1]) > 20:
            logging.warning(f"'{names[1]}' is very long for a short name (>20 chars)")

        building_data["short_name"] = names[1]
    elif len(names) > 2:
        raise RuntimeError(f"Too many names: {names}")
    return building_data
