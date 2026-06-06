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
    if coords := entry.get("coords"):
        row["coords_lat"] = coords.get("lat")
        row["coords_lon"] = coords.get("lon")
        row["coords_source"] = coords.get("source")
        row["coords_accuracy"] = coords.get("accuracy")
        if utm := coords.get("utm"):
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
    if (comment := props.get("comment")) is not None:
        row.update(translatable_to_columns("props_comment", comment))

    # Usage
    if usage := entry.get("usage"):
        row.update(translatable_to_columns("usage_name", usage.get("name")))
        row["usage_din_277"] = usage.get("din_277")
        row["usage_din_277_desc"] = usage.get("din_277_desc")
        if din277_name := usage.get("din277_name"):
            row["usage_din277_name"] = din277_name

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
    row["aliases"] = entry.get("aliases") or []
    row["imgs_json"] = to_json_or_none(entry.get("imgs"))
    if (type_common_name := entry.get("type_common_name")) is not None:
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

    if short_name := row.get("short_name") or row.get("short_name_de"):
        result["short_name"] = short_name
    if visible_id := row.get("visible_id"):
        result["visible_id"] = visible_id

    # b_prefix
    if b_prefix_list := row.get("b_prefix_list"):
        result["b_prefix"] = b_prefix_list
    elif b_prefix := row.get("b_prefix"):
        result["b_prefix"] = b_prefix

    # Coords
    if (lat := row.get("coords_lat")) is not None:
        result["coords"] = {
            "lat": lat,
            "lon": row["coords_lon"],
        }
        if source := row.get("coords_source"):
            result["coords"]["source"] = source
        if accuracy := row.get("coords_accuracy"):
            result["coords"]["accuracy"] = accuracy

    # Props
    props: dict[str, Any] = {}
    ids: dict[str, Any] = {}
    if b_id := row.get("props_ids_b_id"):
        ids["b_id"] = b_id
    if roomcode := row.get("props_ids_roomcode"):
        ids["roomcode"] = roomcode
    if arch_name := row.get("props_ids_arch_name"):
        ids["arch_name"] = arch_name
    if ids:
        props["ids"] = ids

    address: dict[str, Any] = {}
    if street := row.get("props_address_street"):
        address["street"] = street
    if plz_place := row.get("props_address_plz_place"):
        address["plz_place"] = plz_place
    if address_source := row.get("props_address_source"):
        address["source"] = address_source
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
        if (stat := row.get(f"props_stats_{key}")) is not None:
            stats[key] = stat
    if stats:
        props["stats"] = stats

    if operator_code := row.get("props_operator_code"):
        props["operator"] = {
            "code": operator_code,
            "name": {"en": row.get("props_operator_name_en"), "de": row.get("props_operator_name_de")},
            "url": row.get("props_operator_url"),
            "id": row.get("props_operator_id"),
        }

    if calendar_url := row.get("props_calendar_url"):
        props["calendar_url"] = calendar_url
    if (tumonline_room_nr := row.get("props_tumonline_room_nr")) is not None:
        props["tumonline_room_nr"] = tumonline_room_nr
    if floors_json := row.get("props_floors_json"):
        props["floors"] = orjson.loads(floors_json)
    if computed_json := row.get("props_computed_json"):
        props["computed"] = orjson.loads(computed_json)
    if links_json := row.get("props_links_json"):
        props["links"] = orjson.loads(links_json)
    if generic_json := row.get("props_generic_json"):
        props["generic"] = orjson.loads(generic_json)
    if comment_de := row.get("props_comment_de"):
        props["comment"] = {"en": row.get("props_comment_en", ""), "de": comment_de}
    # Emitted only where present, so absent reads as "no coverage" on the info card.
    if row.get("has_iris_coverage"):
        props["has_iris_coverage"] = True

    if props:
        result["props"] = props

    # Usage
    if row.get("usage_name_de") or row.get("usage_din_277"):
        usage: dict[str, Any] = {}
        if name_de := row.get("usage_name_de"):
            usage["name"] = {"en": row.get("usage_name_en"), "de": name_de}
        if din_277 := row.get("usage_din_277"):
            usage["din_277"] = din_277
        if din_277_desc := row.get("usage_din_277_desc"):
            usage["din_277_desc"] = din_277_desc
        if din277_name := row.get("usage_din277_name"):
            usage["din277_name"] = din277_name
        result["usage"] = usage

    # Ranking
    ranking: dict[str, Any] = {}
    for key in ["rank_type", "rank_usage", "rank_boost", "rank_custom", "rank_combined"]:
        if (rank := row.get(f"ranking_{key}")) is not None:
            ranking[key] = rank
    if ranking:
        result["ranking_factors"] = ranking

    # External data
    if tumonline_json := row.get("tumonline_data_json"):
        result["tumonline_data"] = orjson.loads(tumonline_json)
    if roomfinder_json := row.get("roomfinder_data_json"):
        result["roomfinder_data"] = orjson.loads(roomfinder_json)
    if opening_hours_json := row.get("opening_hours_json"):
        result["opening_hours"] = orjson.loads(opening_hours_json)

    # Late-stage
    if arch_name := row.get("arch_name"):
        result["arch_name"] = arch_name
    if aliases := row.get("aliases"):
        result["aliases"] = list(aliases)
    if (imgs_json := row.get("imgs_json")) is not None:
        result["imgs"] = orjson.loads(imgs_json)
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
    if buildings_overview_json := row.get("sections_buildings_overview_json"):
        result.setdefault("sections", {})["buildings_overview"] = orjson.loads(buildings_overview_json)
    if rooms_overview_json := row.get("sections_rooms_overview_json"):
        result.setdefault("sections", {})["rooms_overview"] = orjson.loads(rooms_overview_json)

    # Metadata
    sources: dict[str, Any] = {}
    if sources_base_json := row.get("sources_base_json"):
        sources["base"] = orjson.loads(sources_base_json)
    if sources_patched := row.get("sources_patched"):
        sources["patched"] = sources_patched
    if sources:
        result["sources"] = sources

    if data_quality_json := row.get("data_quality_json"):
        result["data_quality"] = orjson.loads(data_quality_json)
    if generators_json := row.get("generators_json"):
        result["generators"] = orjson.loads(generators_json)

    # Structural (not exported to API, but available)
    if children := row.get("children"):
        result["children"] = children
    if children_flat := row.get("children_flat"):
        result["children_flat"] = children_flat

    # Maps
    if maps_default := row.get("maps_default"):
        result["maps"] = {"default": maps_default}

    # Description
    if description_json := row.get("description_json"):
        result["description"] = orjson.loads(description_json)

    # External data
    if external_data_json := row.get("external_data_json"):
        result["external_data"] = orjson.loads(external_data_json)

    # Custom rooms overview
    if generate_rooms_overview_json := row.get("generate_rooms_overview_json"):
        result["generate_rooms_overview"] = orjson.loads(generate_rooms_overview_json)

    return result
