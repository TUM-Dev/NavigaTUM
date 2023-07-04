import logging


def read_areatree():
    """Reads the areatree file and the basic data, gained from the areatree"""

    data = {}
    parent_stack: list[str] = []

    # The first line is extracted as mypy cannot make sense of this otherwise
    lines = _areatree_lines()
    last_element: str = _parse_areatree_line(next(lines))["id"]
    for line in lines:
        indent = len(line) - len(line.lstrip(" "))
        if indent % 2 != 0:
            raise RuntimeError(f"Indentation not multiple of 2: '{line}'")
        if (indent // 2) > len(parent_stack):
            parent_stack.append(last_element)
        elif (indent // 2) < len(parent_stack):
            parent_stack = parent_stack[: indent // 2]

        building_data = _parse_areatree_line(line)
        last_element = building_data["id"]
        data[building_data["id"]] = building_data | {"parents": parent_stack[:]}
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
    The syntax is building-id(s):name(s):internal-id[,visible-id]
    """
    parts = line.split(":")
    if len(parts) != 3:
        raise RuntimeError(f"Invalid line, expected 3 ':'-separated parts: '{line}'")
    internal_id: str
    raw_names: str
    building_ids: str
    (building_ids, raw_names, internal_id) = parts
    return building_ids.strip(), raw_names.strip(), internal_id.strip()


def _parse_areatree_line(line: str) -> dict:
    """Parses a line from the areatree file to reveal the correct parent and children"""
    (building_ids, raw_names, internal_id) = _split_line(line)

    building_data = _extract_building_prefix(building_ids)
    building_data |= _extract_names(raw_names.split("|"))
    building_data |= _extract_id_and_type(internal_id, line, building_data.get("b_prefix"))

    return building_data


def _extract_id_and_type(internal_id, line, b_prefix: str | list[str] | None):
    """Extracts the id and type from the internal_id"""
    results = {}
    if "[" in internal_id:
        internal_id, results["type"] = internal_id.removesuffix("]").split("[")
    if "," in internal_id:
        ids = internal_id.split(",")
        if len(ids) != 2:
            raise RuntimeError(f"More than two ids found: '{line}'")
        results["id"], results["visible-id"] = ids
    elif internal_id:
        results["id"] = internal_id
    elif isinstance(b_prefix, str) and b_prefix:
        results["id"] = b_prefix
    if "id" not in results:
        raise RuntimeError(f"No id provided in line: '{line}'")
    # we infer which type some elements are, if they have not specified it
    if "type" not in results:
        results["type"] = "building" if results["id"] == b_prefix else "area"
    return results


def _extract_building_prefix(building_ids: str) -> dict:
    """Extracts the building prefix from the building_ids"""
    results = {}
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


def _extract_names(names: list[str]) -> dict[str, str]:
    """Extracts the name and the possible short_name"""
    building_data = {"name": names[0]}
    if len(names) == 2:
        if len(names[1]) > 20:
            logging.warning(f"'{names[1]}' is very long for a short name (>20 chars)")

        building_data["short_name"] = names[1]
    elif len(names) > 2:
        raise RuntimeError(f"Too many names: {names}")
    return building_data
