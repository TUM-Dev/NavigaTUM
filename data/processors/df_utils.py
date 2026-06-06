from typing import Any

import orjson
import polars as pl
from utils import TranslatableStr

_DEFAULT_DTYPE: pl.DataType = pl.Utf8()


def ensure_column(df: pl.DataFrame, col_name: str, dtype: pl.DataType = _DEFAULT_DTYPE) -> pl.DataFrame:
    """Ensure a column exists in the DataFrame, adding it as null if missing."""
    if col_name not in df.columns:
        df = df.with_columns(pl.lit(None).cast(dtype).alias(col_name))
    return df


def ensure_columns(df: pl.DataFrame, columns: dict[str, pl.DataType]) -> pl.DataFrame:
    """Ensure multiple columns exist in the DataFrame, adding them as null if missing."""
    missing = {name: dtype for name, dtype in columns.items() if name not in df.columns}
    if missing:
        df = df.with_columns([pl.lit(None).cast(dtype).alias(name) for name, dtype in missing.items()])
    return df


def translatable_to_columns(field: str, value: Any) -> dict[str, str | None]:
    """Split a TranslatableStr or plain string into _de/_en suffix columns."""
    if value is None:
        return {f"{field}_de": None, f"{field}_en": None}
    if isinstance(value, TranslatableStr):
        return {f"{field}_de": value["de"], f"{field}_en": value["en"]}
    if isinstance(value, dict) and "de" in value:
        return {f"{field}_de": value.get("de"), f"{field}_en": value.get("en")}
    if isinstance(value, str):
        return {f"{field}_de": value, f"{field}_en": value}
    return {f"{field}_de": str(value), f"{field}_en": str(value)}


def to_json_or_none(value: Any) -> str | None:
    """Serialize a value to JSON string, or return None if value is None."""
    if value is None:
        return None
    return orjson.dumps(value).decode()


def flatten_entry(entry_id: str, entry: dict[str, Any]) -> dict[str, Any]:
    """Convert a legacy dict entry to a flat column dict for DataFrame insertion."""
    row: dict[str, Any] = {"id": entry_id}

    # Type
    row["type"] = entry.get("type")

    # Name - can be str or TranslatableStr
    name = entry.get("name")
    if isinstance(name, (TranslatableStr, dict)) and "de" in name:
        row["name"] = name.get("de", name.get("en", ""))
        row.update(translatable_to_columns("name", name))
    elif isinstance(name, str):
        row["name"] = name
        row["name_de"] = name
        row["name_en"] = name
    else:
        row["name"] = name
        row["name_de"] = None
        row["name_en"] = None

    # Short name
    short_name = entry.get("short_name")
    if short_name is not None:
        if isinstance(short_name, str):
            row["short_name"] = short_name
            row["short_name_de"] = short_name
            row["short_name_en"] = short_name
        else:
            row["short_name"] = short_name.get("de") if isinstance(short_name, dict) else str(short_name)
            row.update(translatable_to_columns("short_name", short_name))

    row["visible_id"] = entry.get("visible_id")
    row["parents"] = entry.get("parents", [])

    # b_prefix
    b_prefix = entry.get("b_prefix")
    if isinstance(b_prefix, list):
        row["b_prefix"] = None
        row["b_prefix_list"] = b_prefix
    elif isinstance(b_prefix, str):
        row["b_prefix"] = b_prefix
        row["b_prefix_list"] = None
    else:
        row["b_prefix"] = None
        row["b_prefix_list"] = None

    # Coords
    coords = entry.get("coords", {})
    if coords:
        row["coords_lat"] = coords.get("lat")
        row["coords_lon"] = coords.get("lon")
        row["coords_source"] = coords.get("source")
        row["coords_accuracy"] = coords.get("accuracy")
        utm = coords.get("utm")
        if utm:
            row["coords_utm_easting"] = utm.get("easting")
            row["coords_utm_northing"] = utm.get("northing")
            row["coords_utm_zone_number"] = utm.get("zone_number")
            row["coords_utm_zone_letter"] = utm.get("zone_letter")

    # Props
    props = entry.get("props", {})
    if ids := props.get("ids"):
        row["props_ids_b_id"] = ids.get("b_id")
        row["props_ids_roomcode"] = ids.get("roomcode")
        row["props_ids_arch_name"] = ids.get("arch_name")

    if address := props.get("address"):
        row["props_address_street"] = address.get("street")
        row["props_address_plz_place"] = address.get("plz_place")
        row["props_address_source"] = address.get("source")

    if stats := props.get("stats"):
        row["props_stats_n_rooms"] = stats.get("n_rooms")
        row["props_stats_n_rooms_reg"] = stats.get("n_rooms_reg")
        row["props_stats_n_buildings"] = stats.get("n_buildings")
        row["props_stats_n_seats"] = stats.get("n_seats")
        row["props_stats_n_seats_sitting"] = stats.get("n_seats_sitting")
        row["props_stats_n_seats_standing"] = stats.get("n_seats_standing")
        row["props_stats_n_seats_wheelchair"] = stats.get("n_seats_wheelchair")

    if operator := props.get("operator"):
        row["props_operator_code"] = operator.get("code")
        row.update(translatable_to_columns("props_operator_name", operator.get("name")))
        row["props_operator_url"] = operator.get("url")
        row["props_operator_id"] = operator.get("id")

    row["props_calendar_url"] = props.get("calendar_url")
    row["props_tumonline_room_nr"] = props.get("tumonline_room_nr")
    row["props_floors_json"] = to_json_or_none(props.get("floors"))
    row["props_computed_json"] = to_json_or_none(props.get("computed"))
    row["props_links_json"] = to_json_or_none(props.get("links"))
    row["props_generic_json"] = to_json_or_none(props.get("generic"))
    comment = props.get("comment")
    if comment is not None:
        row.update(translatable_to_columns("props_comment", comment))

    # Usage
    if usage := entry.get("usage"):
        row.update(translatable_to_columns("usage_name", usage.get("name")))
        row["usage_din_277"] = usage.get("din_277")
        row["usage_din_277_desc"] = usage.get("din_277_desc")
        if usage.get("din277_name"):
            row["usage_din277_name"] = usage["din277_name"]

    # Ranking
    if rf := entry.get("ranking_factors"):
        row["ranking_rank_type"] = rf.get("rank_type")
        row["ranking_rank_usage"] = rf.get("rank_usage")
        row["ranking_rank_boost"] = rf.get("rank_boost")
        row["ranking_rank_custom"] = rf.get("rank_custom")
        row["ranking_rank_combined"] = rf.get("rank_combined")

    # External data
    row["tumonline_data_json"] = to_json_or_none(entry.get("tumonline_data"))
    row["roomfinder_data_json"] = to_json_or_none(entry.get("roomfinder_data"))

    # Late-stage
    row["arch_name"] = entry.get("arch_name")
    row["aliases_json"] = to_json_or_none(entry.get("aliases"))
    row["imgs_json"] = to_json_or_none(entry.get("imgs"))
    type_common_name = entry.get("type_common_name")
    if type_common_name is not None:
        if isinstance(type_common_name, str):
            row["type_common_name"] = type_common_name
            row["type_common_name_de"] = type_common_name
            row["type_common_name_en"] = type_common_name
        else:
            row.update(translatable_to_columns("type_common_name", type_common_name))
            row["type_common_name"] = (
                type_common_name.get("de") if isinstance(type_common_name, dict) else str(type_common_name)
            )

    # Sections
    sections = entry.get("sections", {})
    row["sections_buildings_overview_json"] = to_json_or_none(sections.get("buildings_overview"))
    row["sections_rooms_overview_json"] = to_json_or_none(sections.get("rooms_overview"))

    # Metadata
    sources = entry.get("sources", {})
    row["sources_base_json"] = to_json_or_none(sources.get("base"))
    row["sources_patched"] = sources.get("patched")
    row["data_quality_json"] = to_json_or_none(entry.get("data_quality"))
    row["generators_json"] = to_json_or_none(entry.get("generators"))

    # Structural
    row["children"] = entry.get("children")
    row["children_flat"] = entry.get("children_flat")

    # Maps
    row["maps_default"] = entry.get("maps", {}).get("default")

    # Description
    row["description_json"] = to_json_or_none(entry.get("description"))

    # External data
    row["external_data_json"] = to_json_or_none(entry.get("external_data"))

    # Custom rooms overview (only mi has this)
    row["generate_rooms_overview_json"] = to_json_or_none(entry.get("generate_rooms_overview"))

    return row


def unflatten_row(row: dict[str, Any]) -> dict[str, Any]:
    """Reconstruct a nested dict from flat DataFrame columns (for JSON export)."""
    # Name: use TranslatableStr dict when de != en
    name_de = row.get("name_de") or row.get("name")
    name_en = row.get("name_en")
    if name_de and name_en and name_de != name_en:
        name_val: Any = {"en": name_en, "de": name_de}
    else:
        name_val = name_de

    result: dict[str, Any] = {
        "id": row["id"],
        "type": row["type"],
        "name": name_val,
        "parents": row.get("parents", []),
    }

    if row.get("short_name") or row.get("short_name_de"):
        result["short_name"] = row.get("short_name") or row.get("short_name_de")
    if row.get("visible_id"):
        result["visible_id"] = row["visible_id"]

    # b_prefix
    if row.get("b_prefix_list"):
        result["b_prefix"] = row["b_prefix_list"]
    elif row.get("b_prefix"):
        result["b_prefix"] = row["b_prefix"]

    # Coords
    if row.get("coords_lat") is not None:
        result["coords"] = {
            "lat": row["coords_lat"],
            "lon": row["coords_lon"],
        }
        if row.get("coords_source"):
            result["coords"]["source"] = row["coords_source"]
        if row.get("coords_accuracy"):
            result["coords"]["accuracy"] = row["coords_accuracy"]

    # Props
    props: dict[str, Any] = {}
    ids: dict[str, Any] = {}
    if row.get("props_ids_b_id"):
        ids["b_id"] = row["props_ids_b_id"]
    if row.get("props_ids_roomcode"):
        ids["roomcode"] = row["props_ids_roomcode"]
    if row.get("props_ids_arch_name"):
        ids["arch_name"] = row["props_ids_arch_name"]
    if ids:
        props["ids"] = ids

    address: dict[str, Any] = {}
    if row.get("props_address_street"):
        address["street"] = row["props_address_street"]
    if row.get("props_address_plz_place"):
        address["plz_place"] = row["props_address_plz_place"]
    if row.get("props_address_source"):
        address["source"] = row["props_address_source"]
    if address:
        props["address"] = address

    stats: dict[str, Any] = {}
    for key in [
        "n_rooms",
        "n_rooms_reg",
        "n_buildings",
        "n_seats",
        "n_seats_sitting",
        "n_seats_standing",
        "n_seats_wheelchair",
    ]:
        if row.get(f"props_stats_{key}") is not None:
            stats[key] = row[f"props_stats_{key}"]
    if stats:
        props["stats"] = stats

    if row.get("props_operator_code"):
        props["operator"] = {
            "code": row["props_operator_code"],
            "name": {"en": row.get("props_operator_name_en"), "de": row.get("props_operator_name_de")},
            "url": row.get("props_operator_url"),
            "id": row.get("props_operator_id"),
        }

    if row.get("props_calendar_url"):
        props["calendar_url"] = row["props_calendar_url"]
    if row.get("props_tumonline_room_nr") is not None:
        props["tumonline_room_nr"] = row["props_tumonline_room_nr"]
    if row.get("props_floors_json"):
        props["floors"] = orjson.loads(row["props_floors_json"])
    if row.get("props_computed_json"):
        props["computed"] = orjson.loads(row["props_computed_json"])
    if row.get("props_links_json"):
        props["links"] = orjson.loads(row["props_links_json"])
    if row.get("props_generic_json"):
        props["generic"] = orjson.loads(row["props_generic_json"])
    if row.get("props_comment_de"):
        props["comment"] = {"en": row.get("props_comment_en", ""), "de": row["props_comment_de"]}
    # Emitted only where present, so absent reads as "no coverage" on the info card.
    if row.get("has_iris_coverage"):
        props["has_iris_coverage"] = True

    if props:
        result["props"] = props

    # Usage
    if row.get("usage_name_de") or row.get("usage_din_277"):
        usage: dict[str, Any] = {}
        if row.get("usage_name_de"):
            usage["name"] = {"en": row.get("usage_name_en"), "de": row["usage_name_de"]}
        if row.get("usage_din_277"):
            usage["din_277"] = row["usage_din_277"]
        if row.get("usage_din_277_desc"):
            usage["din_277_desc"] = row["usage_din_277_desc"]
        if row.get("usage_din277_name"):
            usage["din277_name"] = row["usage_din277_name"]
        result["usage"] = usage

    # Ranking
    ranking: dict[str, Any] = {}
    for key in ["rank_type", "rank_usage", "rank_boost", "rank_custom", "rank_combined"]:
        if row.get(f"ranking_{key}") is not None:
            ranking[key] = row[f"ranking_{key}"]
    if ranking:
        result["ranking_factors"] = ranking

    # External data
    if row.get("tumonline_data_json"):
        result["tumonline_data"] = orjson.loads(row["tumonline_data_json"])
    if row.get("roomfinder_data_json"):
        result["roomfinder_data"] = orjson.loads(row["roomfinder_data_json"])

    # Late-stage
    if row.get("arch_name"):
        result["arch_name"] = row["arch_name"]
    if row.get("aliases_json"):
        result["aliases"] = orjson.loads(row["aliases_json"])
    if row.get("imgs_json") is not None:
        result["imgs"] = orjson.loads(row["imgs_json"])
    if row.get("type_common_name") or row.get("type_common_name_de"):
        tcn_de = row.get("type_common_name_de") or row.get("type_common_name")
        tcn_en = row.get("type_common_name_en") or tcn_de
        # Values from usage_name are always TranslatableStr dicts (even when de==en).
        # Values from TYPE_COMMON_NAME_BY_TYPE that were plain strings stay as strings.
        plain_string_types = {"Campus", "POI"}
        if tcn_de in plain_string_types and tcn_de == tcn_en:
            result["type_common_name"] = tcn_de
        else:
            result["type_common_name"] = {"en": tcn_en, "de": tcn_de}

    # Sections
    if row.get("sections_buildings_overview_json"):
        result.setdefault("sections", {})["buildings_overview"] = orjson.loads(row["sections_buildings_overview_json"])
    if row.get("sections_rooms_overview_json"):
        result.setdefault("sections", {})["rooms_overview"] = orjson.loads(row["sections_rooms_overview_json"])

    # Metadata
    sources: dict[str, Any] = {}
    if row.get("sources_base_json"):
        sources["base"] = orjson.loads(row["sources_base_json"])
    if row.get("sources_patched"):
        sources["patched"] = row["sources_patched"]
    if sources:
        result["sources"] = sources

    if row.get("data_quality_json"):
        result["data_quality"] = orjson.loads(row["data_quality_json"])
    if row.get("generators_json"):
        result["generators"] = orjson.loads(row["generators_json"])

    # Structural (not exported to API, but available)
    if row.get("children"):
        result["children"] = row["children"]
    if row.get("children_flat"):
        result["children_flat"] = row["children_flat"]

    # Maps
    if row.get("maps_default"):
        result["maps"] = {"default": row["maps_default"]}

    # Description
    if row.get("description_json"):
        result["description"] = orjson.loads(row["description_json"])

    # External data
    if row.get("external_data_json"):
        result["external_data"] = orjson.loads(row["external_data_json"])

    # Custom rooms overview
    if row.get("generate_rooms_overview_json"):
        result["generate_rooms_overview"] = orjson.loads(row["generate_rooms_overview_json"])

    return result
