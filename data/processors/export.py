import json
import re
from pathlib import Path
from typing import Any
import polars as pl

from external.models.common import PydanticConfiguration
from utils import TranslatableStr
from utils import TranslatableStr as _

OUTPUT_DIR_PATH = Path(__file__).parent.parent / "output"
OUTPUT_DIR_PATH.mkdir(exist_ok=True)
SLUGIFY_REGEX = re.compile(r"[^a-zA-Z0-9-äöüß.]+")


def maybe_slugify(value: str | None | TranslatableStr) -> str | None:
    """Slugify a value if it exists"""
    if value is None:
        return None
    if isinstance(value, TranslatableStr):
        value = unlocalise(value)

    if not isinstance(value, str):
        raise ValueError(f"Expected str, got {type(value)}")
    return SLUGIFY_REGEX.sub("-", value.lower()).strip("-")


def unlocalise(value: str | list[Any] | dict[str, Any]) -> Any:
    """Recursively unlocalise a dictionary"""
    if isinstance(value, bool | float | int | str) or value is None:
        return value
    if isinstance(value, list):
        return [unlocalise(v) for v in value]
    if isinstance(value, dict):
        # We consider each dict that has only the keys "de" and/or "en" as translated string
        if set(value.keys()) | {"de", "en"} == {"de", "en"}:
            # Since we only unlocalise dicts with either en and/or de or {}, the default to {} is fine
            return value.get("de", value.get("en", {}))

        return {k: unlocalise(v) for k, v in value.items()}
    raise ValueError(f"Unhandled type {type(value)}")


def normalise_id(_id: str) -> str | None:
    """Remove leading zeros from all point-separated parts of input string"""
    if not _id:
        return None

    parts = [part.lstrip("0") or "0" for part in _id.split(".")]
    return ".".join(parts)


def export_for_search(data: dict) -> None:
    """Export a subset of the data for the /search api"""
    export = []
    for _id, entry in data.items():
        building_parents_index = len(entry["parents"])
        if entry["type"] in {"room", "virtual_room"}:
            for i, parent in enumerate(entry["parents"]):
                if parent == "root":
                    continue
                if data[parent]["type"] in {"building", "joined_building"}:
                    building_parents_index = i
                    break

        # The 'campus name' is the campus of site of this building or room
        campus_name = None
        if entry["type"] not in {"campus", "site"}:
            for parent in entry["parents"]:
                if parent == "root":
                    continue
                if data[parent]["type"] in {"campus", "site"}:
                    campus = data[parent]
                    campus_name = campus.get("short_name", campus["name"])
                    # intentionally no break, because sites might be below a campus

        geo = {}
        if coords := entry.get("coords"):
            geo["_geo"] = {"lat": coords["lat"], "lng": coords["lon"]}
        parent_building_names = extract_parent_building_names(data, entry["parents"], building_parents_index)
        address = entry.get("tumonline_data", {}).get("address", {})
        export.append(
            {
                # MeiliSearch requires an id without "."
                # also this puts more emphasis on the order (because "." counts as more distance)
                "ms_id": _id.replace(".", "-"),
                "room_code": _id,
                "room_code_normalised": normalise_id(_id),
                "name": entry["name"],
                "arch_name": entry.get("arch_name"),
                "arch_name_normalised": normalise_id(entry.get("arch_name")),
                "type": entry["type"],
                "type_common_name": entry["type_common_name"],
                "facet": {
                    "site": "site",
                    "campus": "site",
                    "area": "site",
                    "joined_building": "building",
                    "building": "building",
                    "room": "room",
                    "virtual_room": "room",
                }.get(entry["type"]),
                "operator_name": entry["props"].get("operator", {}).get("name", None),
                "parent_building_names": parent_building_names,
                # For all other parents, only the ids and their keywords (TODO) are searchable
                "parent_keywords": [maybe_slugify(value) for value in parent_building_names + entry["parents"][1:]],
                "campus": maybe_slugify(campus_name),
                "address": address.get("street", None) if isinstance(address, dict) else address.street,
                "usage": maybe_slugify(entry.get("usage", {}).get("name", None)),
                "rank": int(entry["ranking_factors"]["rank_combined"]),
                **geo,
            },
        )

    # the data contains translations, currently we don't allow these in the search api
    export = unlocalise(export)

    _make_sure_is_safe(export)
    with (OUTPUT_DIR_PATH / "search_data.json").open("w+", encoding="UTF-8") as file:
        json.dump(export, file)
    df = pl.read_json(OUTPUT_DIR_PATH / "search_data.json")
    df.write_parquet(OUTPUT_DIR_PATH / "search_data.parquet", use_pyarrow=True, compression_level=22)


def extract_parent_building_names(data: dict, parents: list[str], building_parents_index: int) -> list[str]:
    """Extract the parents building names from the data"""
    # For rooms, the (joined_)building parents are extra to put more emphasis on them.
    short_names = [data[p]["short_name"] for p in parents[building_parents_index:] if "short_name" in data[p]]
    long_names = [data[p]["name"] for p in parents[building_parents_index:]]
    return short_names + long_names


def _make_sure_is_safe(obj: object):
    """
    Check if any of the specified names in removed_names are present

    :param obj: obj to be checked
    :raises RuntimeError: If any of the specified names (removed_names) are found in the content of the file.
    """
    removed_names = ["bestelmeyer", "gustav niemann", "prandtl", "messerschmidt"]
    allowed_variation = "prandtl str"
    if isinstance(obj, str):
        content = obj.lower().replace("  ", " ").replace("-", " ")
        for name in removed_names:
            if name in content and allowed_variation not in content:
                raise RuntimeError(
                    f"{name} was purposely renamed due to NS context. Please make sure it is not included"
                )
    elif isinstance(obj, dict):
        for key, val in obj.items():
            _make_sure_is_safe(key)
            _make_sure_is_safe(val)
    elif isinstance(obj, list) or isinstance(obj, tuple):
        for entry in obj:
            _make_sure_is_safe(entry)
    elif isinstance(obj, PydanticConfiguration):
        return _make_sure_is_safe(obj.model_dump())
    elif isinstance(obj, bool) or isinstance(obj, int) or isinstance(obj, float) or obj is None:
        pass
    else:
        raise ValueError(f"unhandled type: {type(obj)}")


def export_for_status() -> None:
    """Generate hashes for the contents of data"""
    with (OUTPUT_DIR_PATH / "api_data.json").open(encoding="utf-8") as file:
        export_data = json.load(file)
    export_json_data = [(d["id"], d["hash"]) for d in export_data]
    with (OUTPUT_DIR_PATH / "status_data.json").open("w", encoding="utf-8") as file:
        json.dump(export_json_data, file)

    export_polars_data = [{"id": d["id"], "hash": d["hash"]} for d in export_data]
    df = pl.DataFrame(export_polars_data)
    df.write_parquet(OUTPUT_DIR_PATH / "status_data.parquet", use_pyarrow=True, compression_level=22)


def export_for_api(data: dict) -> None:
    """Add some more information about parents to the data and export for the /locations/:id api"""
    export_data = []
    for _id, entry in data.items():
        entry.setdefault("maps", {})["default"] = "interactive"
        export_data.append(extract_exported_item(data, entry))

    _make_sure_is_safe(export_data)
    with (OUTPUT_DIR_PATH / "api_data.json").open("w", encoding="utf-8") as file:
        json.dump(export_data, file, cls=EnhancedJSONEncoder)
    df = pl.read_json(OUTPUT_DIR_PATH / "api_data.json")
    df.write_parquet(OUTPUT_DIR_PATH / "api_data.parquet", use_pyarrow=True, compression_level=22)


def extract_exported_item(data, entry):
    """Extract the item that will be finally exported to the api"""
    parent_names = [data[p]["name"] if not p == "root" else _("Standorte", "Sites") for p in entry["parents"]]
    result = {
        "parent_names": parent_names,
        **entry,
    }
    if "children" in result:
        del result["children"]
        del result["children_flat"]
    for key in ["tumonline_data", "roomfinder_data", "nat_data"]:
        result.pop(key, None)
    if "props" in result:
        prop_keys_to_keep = {"computed", "links", "comment", "calendar_url", "tumonline_room_nr", "operator"}
        to_delete = [e for e in result["props"].keys() if e not in prop_keys_to_keep]
        for k in to_delete:
            del result["props"][k]
    result["hash"] = hash(json.dumps(result, sort_keys=True, cls=EnhancedJSONEncoder))
    return result


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o: Any) -> Any:
        """Enhanced JSONEncoder that can handle dataclasses"""
        if isinstance(o, PydanticConfiguration):
            return o.model_dump()
        return super().default(o)
