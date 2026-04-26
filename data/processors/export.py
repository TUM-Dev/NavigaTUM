import re
from pathlib import Path
from typing import Any
import orjson
import polars as pl
import csv
import yaml

from external.models import tumonline
from external.models.common import PydanticConfiguration
from processors.df_utils import unflatten_row
from utils import TranslatableStr
from utils import TranslatableStr as _


def _orjson_default(o: Any) -> Any:
    if isinstance(o, PydanticConfiguration):
        return o.model_dump()
    raise TypeError(f"Object of type {type(o)} is not JSON serializable")

OUTPUT_DIR_PATH = Path(__file__).parent.parent / "output"
OUTPUT_DIR_PATH.mkdir(exist_ok=True)
SLUGIFY_REGEX = re.compile(r"[^a-zA-Z0-9-äöüß.]+")

_REMOVED_NAMES_RE = re.compile(rb"bestelmeyer|gustav[ -]+niemann|prandtl|messerschmidt", re.IGNORECASE)
_ALLOWED_VARIATION_RE = re.compile(rb"prandtl[ -]+str", re.IGNORECASE)


def _de(value: Any) -> Any:
    """Pick the German variant from a TranslatableStr-shaped dict; pass-through otherwise."""
    if isinstance(value, dict) and value.keys() <= {"de", "en"}:
        return value.get("de", value.get("en", {}))
    return value


def maybe_slugify(value: str | None | TranslatableStr | dict[str, Any]) -> str | None:
    """Slugify a value if it exists"""
    if value is None:
        return None
    value = _de(value)
    if not isinstance(value, str):
        raise ValueError(f"Expected str, got {type(value)}")
    return SLUGIFY_REGEX.sub("-", value.lower()).strip("-")


def normalise_id(_id: str) -> str | None:
    """Remove leading zeros from all point-separated parts of input string"""
    if not _id:
        return None

    parts = [part.lstrip("0") or "0" for part in _id.split(".")]
    return ".".join(parts)


def reconstruct_data(df: pl.DataFrame) -> dict[str, Any]:
    """Reconstruct nested data dict from flat DataFrame (shared by search and API export)."""
    data = {}
    for row in df.to_dicts():
        entry = unflatten_row(row)
        data[entry["id"]] = entry
    return data


def export_for_search(data: dict[str, Any]) -> None:
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
        parent_building_names = [_de(n) for n in extract_parent_building_names(data, entry["parents"], building_parents_index)]
        address = entry.get("tumonline_data", {}).get("address", {})
        street = address.get("street", None) if isinstance(address, dict) else address.street
        export.append(
            {
                # MeiliSearch requires an id without "."
                # also this puts more emphasis on the order (because "." counts as more distance)
                "ms_id": _id.replace(".", "-"),
                "room_code": _id,
                "room_code_normalised": normalise_id(_id),
                "name": _de(entry["name"]),
                "arch_name": entry.get("arch_name"),
                "arch_name_normalised": normalise_id(entry.get("arch_name", "")),
                "type": entry["type"],
                "type_common_name": _de(entry["type_common_name"]),
                "facet": {
                    "site": "site",
                    "campus": "site",
                    "area": "site",
                    "joined_building": "building",
                    "building": "building",
                    "room": "room",
                    "virtual_room": "room",
                }.get(entry["type"]),
                "operator_name": _de(entry.get("props", {}).get("operator", {}).get("name", None)),
                "parent_building_names": parent_building_names,
                # For all other parents, only the ids and their keywords (TODO) are searchable
                "parent_keywords": [maybe_slugify(value) for value in parent_building_names + entry["parents"][1:]],
                "campus": maybe_slugify(campus_name),
                "address": _de(street),
                "usage": maybe_slugify(entry.get("usage", {}).get("name", None)),
                "rank": int(entry["ranking_factors"]["rank_combined"]),
                **geo,
            },
        )

    search_bytes = orjson.dumps(export)
    _make_sure_is_safe(search_bytes)
    (OUTPUT_DIR_PATH / "search_data.json").write_bytes(search_bytes)
    search_df = pl.DataFrame(export, infer_schema_length=None)
    search_df.write_parquet(OUTPUT_DIR_PATH / "search_data.parquet", use_pyarrow=True, compression_level=22)


def extract_parent_building_names(data: dict[str, Any], parents: list[str], building_parents_index: int) -> list[str]:
    """Extract the parents building names from the data"""
    # For rooms, the (joined_)building parents are extra to put more emphasis on them.
    short_names = [data[p]["short_name"] for p in parents[building_parents_index:] if "short_name" in data[p]]
    long_names = [data[p]["name"] for p in parents[building_parents_index:]]
    return short_names + long_names


def _make_sure_is_safe(blob: bytes) -> None:
    """
    Check that no NS-context names slipped into the export.

    :param blob: serialized JSON bytes to be checked
    :raises RuntimeError: if a forbidden name is found.
    """
    for match in _REMOVED_NAMES_RE.finditer(blob):
        if not _ALLOWED_VARIATION_RE.match(blob, match.start()):
            raise RuntimeError(
                f"{match.group().decode()} was purposely renamed due to NS context. Please make sure it is not included",
            )


def export_for_status() -> None:
    """Generate hashes for the contents of data"""
    export_data = orjson.loads((OUTPUT_DIR_PATH / "api_data.json").read_bytes())
    export_json_data = [(d["id"], d["hash"]) for d in export_data]
    (OUTPUT_DIR_PATH / "status_data.json").write_bytes(orjson.dumps(export_json_data))

    export_polars_data = [{"id": d["id"], "hash": d["hash"]} for d in export_data]
    df = pl.DataFrame(export_polars_data, infer_schema_length=None)
    df.write_parquet(OUTPUT_DIR_PATH / "status_data.parquet", use_pyarrow=True, compression_level=22)


def export_for_api(data: dict[str, Any]) -> None:
    """Add some more information about parents to the data and export for the /locations/:id api"""
    export_data = []
    for entry in data.values():
        entry.setdefault("maps", {})["default"] = "interactive"
        export_data.append(extract_exported_item(data, entry))

    api_data_bytes = orjson.dumps(export_data, default=_orjson_default)
    _make_sure_is_safe(api_data_bytes)
    (OUTPUT_DIR_PATH / "api_data.json").write_bytes(api_data_bytes)
    alias_data = [{k: r.get(k) for k in ("id", "type", "visible_id", "aliases")} for r in orjson.loads(api_data_bytes)]
    df = pl.DataFrame(alias_data, infer_schema_length=None)
    df.write_parquet(OUTPUT_DIR_PATH / "alias_data.parquet", use_pyarrow=True, compression_level=22)


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
        prop_keys_to_keep = {"computed", "floors", "links", "comment", "calendar_url", "tumonline_room_nr", "operator"}
        to_delete = [e for e in result["props"].keys() if e not in prop_keys_to_keep]
        for k in to_delete:
            del result["props"][k]
    result["hash"] = hash(orjson.dumps(result, option=orjson.OPT_SORT_KEYS, default=_orjson_default))
    return result


def export_known_usages(df: pl.DataFrame) -> None:
    """Export the known room usages (categories) for the frontend feedback dropdown."""
    data_dir = Path(__file__).parent.parent
    translations = yaml.safe_load((data_dir / "translations.yaml").read_text(encoding="utf-8"))

    usages_df = (
        pl.read_csv(
            data_dir / "external" / "results" / "usages_tumonline.csv",
            schema_overrides={"din277_id": pl.String, "name": pl.String},
        )
        .select(
            pl.col("name").alias("name_de"),
            pl.col("din277_id").alias("din_277"),
        )
        .unique()
    )

    counts_df = (
        df.filter(pl.col("usage_name_de").is_not_null() & pl.col("usage_din_277").is_not_null())
        .group_by("usage_name_de", "usage_din_277")
        .len()
    )

    result_df = (
        usages_df.join(
            counts_df,
            left_on=["name_de", "din_277"],
            right_on=["usage_name_de", "usage_din_277"],
            how="left",
        )
        .with_columns(
            pl.col("name_de").replace_strict(translations, default=pl.col("name_de")).alias("name_en"),
            pl.col("len").fill_null(0).alias("occurrences"),
        )
        .select("name_de", "name_en", "din_277", "occurrences")
        .sort("occurrences", descending=True)
    )

    (OUTPUT_DIR_PATH / "known_usages.json").write_bytes(
        orjson.dumps(result_df.to_dicts(), option=orjson.OPT_INDENT_2) + b"\n"
    )


