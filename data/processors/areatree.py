def read_areatree():
    """Reads the areatree file and the basic data, gained from the areatree"""

    data = {}
    parent_stack = []
    last_element = None
    with open("sources/00_areatree", encoding="utf-8") as file:
        for line in file.readlines():
            # Empty lines and comment lines are ignored
            line = line.split("#")[0]
            if len(line.strip()) == 0:
                continue

            indent = len(line) - len(line.lstrip(" "))
            if indent % 2 != 0:
                raise RuntimeError(f"Indentation not multiple of 2: '{line}'")
            if (indent // 2) > len(parent_stack):
                parent_stack.append(last_element)
            elif (indent // 2) < len(parent_stack):
                parent_stack = parent_stack[: indent // 2]

            last_element = _parse_areatree_line(line, parent_stack, data)
    return data


def _parse_areatree_line(line, parent_stack, data):
    """Parses a line from the areatree file to reveal the correct parent and children"""

    # The syntax is building-id(s):name(s):internal-id[,visible-id]
    parts = line.split(":")
    if len(parts) != 3:
        raise RuntimeError(f"Invalid line, expected 3 ':'-separated parts: '{line}'")

    building_data = {
        "parents": parent_stack[:],
    }

    # building id(s)
    if "-" in parts[0]:
        building_data["data_quality"] = {"areatree_uncertain": True}
        parts[0] = parts[0].replace("-", "")

    if "," in parts[0]:
        building_data["b_prefix"] = parts[0].strip().split(",")
    elif len(parts[0].strip()) > 0:
        b_id = parts[0].strip()
        building_data["b_prefix"] = b_id
        building_data["id"] = b_id

    # name
    building_data["name"] = parts[1]

    # id and type
    if "[" in parts[2]:
        building_data["type"] = parts[2].split("[")[1].strip()[:-1]
        parts[2] = parts[2].split("[")[0]

    if "," in parts[2]:
        ids = parts[2].strip().split(",")
        if len(ids) != 2:
            raise RuntimeError(f"More than two ids found: '{line}'")

        building_data["id"] = ids[0]
        building_data["visible-id"] = ids[1]
    elif len(parts[2].strip()) > 0:
        building_data["id"] = parts[2].strip()

    if "id" not in building_data:
        raise RuntimeError(f"No id provided in line: '{line}'")

    # we infer which type some elements are, if they have not specified it
    if "type" not in building_data:
        if "b_prefix" in building_data and building_data["id"] == building_data["b_prefix"]:
            building_data["type"] = "building"
        else:
            building_data["type"] = "area"

    data[building_data["id"]] = building_data
    return building_data["id"]
