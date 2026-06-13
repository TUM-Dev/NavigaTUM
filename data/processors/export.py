import re
from collections.abc import Mapping
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path
from typing import TypedDict

import dataframely as dy
import orjson
import polars as pl
import xxhash
import yaml
from external.loaders.events import load_events
from external.loaders.tumonline_orgs import load_tumonline_orgs
from external.models.common import PydanticConfiguration
from external.schemas.events import EventsSchema
from external.schemas.tumonline_orgs import TumonlineOrgsSchema
from pipeline_types import Entry, Json
from utils import TranslatableStr
from utils import TranslatableStr as _

from processors.df_utils import unflatten_row
from processors.images import ImageOffset, ImageSource


def _orjson_default(o: Json) -> Json:
    if isinstance(o, PydanticConfiguration):
        return o.model_dump()
    raise TypeError(f"Object of type {type(o)} is not JSON serializable")


class SearchFacet(StrEnum):
    """
    The search facet a document is bucketed into by the search API.

    The closed set the server's `facet` field is matched against. Locations map
    via `_TYPE_TO_FACET`; types not listed there (the synthetic `root` node)
    have no facet and are not locatable results, so they are skipped at export
    rather than indexed. Events come from `events.csv` instead of the location
    tree.
    """

    SITE = "site"
    BUILDING = "building"
    ROOM = "room"
    POI = "poi"
    EVENT = "event"


# Location `type` -> the search facet it surfaces under.
_TYPE_TO_FACET: dict[str, SearchFacet] = {
    "site": SearchFacet.SITE,
    "campus": SearchFacet.SITE,
    "area": SearchFacet.SITE,
    "joined_building": SearchFacet.BUILDING,
    "building": SearchFacet.BUILDING,
    "room": SearchFacet.ROOM,
    "virtual_room": SearchFacet.ROOM,
    "poi": SearchFacet.POI,
}
# Types that deliberately carry no search facet and are excluded from the index.
# Only the synthetic tree root - everything else must map to a facet, so an
# unmapped type is a bug (a new type was added without deciding how it searches).
_NON_SEARCHABLE_TYPES: frozenset[str] = frozenset({"root"})

# The `event_<hash>` identity is the base name of the key-named image files.
_EVENT_IMAGE_KEY_REGEX = re.compile(r"^/cdn/thumb/(?P<key>.+)_\d+\.webp$")


class _GeoPoint(TypedDict):
    """Meilisearch's `_geo` shape (note `lng`, not `lon`)."""

    lat: float
    lng: float


class EventSearchDocument(TypedDict):
    """One `events.csv` row as a search document for the `event` facet."""

    ms_id: str
    facet: str
    name: str
    starts_at: str
    ends_at: str
    description: str
    organising_org_id: int
    image: str
    image_author: str
    # Image crop offsets (pixel shift of the crop window along each image's longer axis), so a client can recover the crop.
    image_thumb_offset: int
    image_header_offset: int
    rank: int
    _geo: _GeoPoint


def _utc_rfc3339(value: str) -> str:
    """Normalise an RFC 3339 timestamp to second-precision UTC (`…Z`)."""
    return f"{datetime.fromisoformat(value).astimezone(UTC):%Y-%m-%dT%H:%M:%SZ}"


def event_search_documents(
    events: dy.DataFrame[EventsSchema],
    image_sources: Mapping[str, list[ImageSource]],
) -> list[EventSearchDocument]:
    """Build one search document per events row for the default-disabled `event` facet."""
    docs: list[EventSearchDocument] = []
    for row in events.iter_rows(named=True):
        match = _EVENT_IMAGE_KEY_REGEX.match(row["image"])
        if match is None:
            raise ValueError(f"event image {row['image']!r} does not contain an extractable event key")
        # An event has a single image; a key with no source entry crops at the unshifted (0) default.
        sources = image_sources.get(match["key"])
        offsets = sources[0].offsets if sources else ImageOffset()
        docs.append(
            {
                # The addition key is the event's identity: Meilisearch upserts by `ms_id`,
                # so a resubmitted key replaces the document instead of adding a second one.
                "ms_id": match["key"],
                "facet": SearchFacet.EVENT.value,
                "name": row["name"],
                "starts_at": _utc_rfc3339(row["starts_at"]),
                "ends_at": _utc_rfc3339(row["ends_at"]),
                "description": row["description"],
                "organising_org_id": row["organising_org_id"],
                "image": row["image"],
                "image_author": row["image_author"],
                "image_thumb_offset": offsets.thumb,
                "image_header_offset": offsets.header,
                # Lecture precedent: uniform 0 keeps the `rank:desc` ranking rule neutral.
                "rank": 0,
                "_geo": {"lat": row["lat"], "lng": row["lon"]},
            },
        )
    return docs


OUTPUT_DIR_PATH = Path(__file__).parent.parent / "output"
OUTPUT_DIR_PATH.mkdir(exist_ok=True)
SLUGIFY_REGEX = re.compile(r"[^a-zA-Z0-9-äöüß.]+")

_REMOVED_NAMES_RE = re.compile(rb"bestelmeyer|gustav[ -]+niemann|prandtl|messerschmidt", re.IGNORECASE)
_ALLOWED_VARIATION_RE = re.compile(rb"prandtl[ -]+str", re.IGNORECASE)


def _de(value: Json) -> Json:
    """Pick the German variant from a TranslatableStr-shaped dict; pass-through otherwise."""
    if isinstance(value, dict) and value.keys() <= {"de", "en"}:
        return value.get("de", value.get("en", {}))
    return value


def maybe_slugify(value: str | None | TranslatableStr | dict[str, Json]) -> str | None:
    """Slugify a value if it exists"""
    if value is None:
        return None
    value = _de(value)
    if not isinstance(value, str):
        raise TypeError(f"Expected str, got {type(value)}")
    return SLUGIFY_REGEX.sub("-", value.lower()).strip("-")


def normalise_id(_id: str) -> str | None:
    """Remove leading zeros from all point-separated parts of input string"""
    if not _id:
        return None

    parts = [part.lstrip("0") or "0" for part in _id.split(".")]
    return ".".join(parts)


def reconstruct_data(df: pl.DataFrame) -> dict[str, Entry]:
    """Reconstruct nested data dict from flat DataFrame (shared by search and API export)."""
    data = {}
    for row in df.to_dicts():
        entry = unflatten_row(row)
        data[entry["id"]] = entry
    return data


def export_for_search(data: dict[str, Entry]) -> None:
    """Export a subset of the data for the /search api"""
    export: list[Mapping[str, Json]] = []
    for _id, entry in data.items():
        facet = _TYPE_TO_FACET.get(entry["type"])
        if facet is None:
            if entry["type"] in _NON_SEARCHABLE_TYPES:
                # The synthetic `root` node is not a locatable result.
                continue
            raise ValueError(
                f"location {_id!r} has type {entry['type']!r}, which maps to no search facet; "
                f"add it to _TYPE_TO_FACET or _NON_SEARCHABLE_TYPES",
            )
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
        parent_building_names = [
            _de(n) for n in extract_parent_building_names(data, entry["parents"], building_parents_index)
        ]
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
                "aliases": entry.get("aliases", []),
                "type": entry["type"],
                "type_common_name": _de(entry["type_common_name"]),
                "facet": facet.value,
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

    export.extend(event_search_documents(load_events(), ImageSource.load_all()))

    search_bytes = orjson.dumps(export)
    _make_sure_is_safe(search_bytes)
    (OUTPUT_DIR_PATH / "search_data.json").write_bytes(search_bytes)
    search_df = pl.DataFrame(export, infer_schema_length=None)
    search_df.write_parquet(OUTPUT_DIR_PATH / "search_data.parquet", use_pyarrow=True, compression_level=3)


def extract_parent_building_names(data: dict[str, Entry], parents: list[str], building_parents_index: int) -> list[str]:
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
    df.write_parquet(OUTPUT_DIR_PATH / "status_data.parquet", use_pyarrow=True, compression_level=3)


def export_for_api(data: dict[str, Entry]) -> None:
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
    df.write_parquet(OUTPUT_DIR_PATH / "alias_data.parquet", use_pyarrow=True, compression_level=3)


def extract_exported_item(data: dict[str, Entry], entry: Entry) -> Entry:
    """Extract the item that will be finally exported to the api"""
    parent_names = [data[p]["name"] if p != "root" else _("Standorte", "Sites") for p in entry["parents"]]
    # Parallel to `parents`/`parent_names`: each parent's type lets the client build the canonical
    # /{type}/{id} breadcrumb link without a per-id round-trip. `root` is synthetic (no data entry).
    parent_types = [data[p]["type"] if p != "root" else "root" for p in entry["parents"]]
    result = {
        "parent_names": parent_names,
        "parent_types": parent_types,
        **entry,
    }
    if "children" in result:
        del result["children"]
        del result["children_flat"]
    for key in ["tumonline_data", "roomfinder_data", "nat_data"]:
        result.pop(key, None)
    if "props" in result:
        prop_keys_to_keep = {
            "computed",
            "floors",
            "links",
            "comment",
            "calendar_url",
            "tumonline_room_nr",
            "operator",
            "iris_coverage_building_ids",
        }
        to_delete = [e for e in result["props"] if e not in prop_keys_to_keep]
        for k in to_delete:
            del result["props"][k]
    # Stable, deterministic content hash. Python's built-in `hash()` is salted by PYTHONHASHSEED
    # and varies across processes, which makes it useless as a cache/etag fingerprint. xxhash is
    # a fast non-cryptographic hash; xxh64 returns a 64-bit value we coerce into a signed int64
    # (same value range as the prior `hash()`, drop-in for the parquet/JSON consumers).
    serialised = orjson.dumps(result, option=orjson.OPT_SORT_KEYS, default=_orjson_default)
    digest = xxhash.xxh64(serialised).intdigest()
    result["hash"] = digest - (1 << 64) if digest >= (1 << 63) else digest
    return result


def export_known_usages(df: pl.DataFrame) -> None:
    """Export the known room usages (categories) for the frontend feedback dropdown."""
    data_dir = Path(__file__).parent.parent
    translations = yaml.safe_load((data_dir / "translations.yaml").read_text(encoding="utf-8"))

    usages_df = pl.read_csv(
        data_dir / "external" / "results" / "usages_tumonline.csv",
        schema_overrides={"din277_id": pl.String, "name": pl.String},
    ).select(
        pl.col("usage_id"),
        pl.col("name").alias("name_de"),
        pl.col("din277_id").alias("din_277"),
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
        .select("usage_id", "name_de", "name_en", "din_277", "occurrences")
        .sort(["occurrences", "name_de"], descending=[True, False])
    )

    (OUTPUT_DIR_PATH / "known_usages.json").write_bytes(
        orjson.dumps(result_df.to_dicts(), option=orjson.OPT_INDENT_2) + b"\n"
    )


def export_tumonline_orgs_parquet() -> None:
    """Build the bilingual TUMonline orgs frame and write tumonline_orgs.parquet."""
    TumonlineOrgsSchema.write_parquet(load_tumonline_orgs(), OUTPUT_DIR_PATH / "tumonline_orgs.parquet")


def export_known_orgs() -> None:
    """Export the known TUMonline orgs as json"""
    # `org_id` is the value submitted as `events.organising_org_id`
    # `code` is the human-readable disambiguator for the ~100 orgs that share a localized name
    result_df = load_tumonline_orgs().select(["org_id", "code", "name_de", "name_en"]).sort("code")

    (OUTPUT_DIR_PATH / "known_orgs.json").write_bytes(
        orjson.dumps(result_df.to_dicts(), option=orjson.OPT_INDENT_2) + b"\n"
    )


def export_events_parquet() -> None:
    """
    Read events.csv and write events.parquet.

    Datetimes stay as ISO-8601 strings so the Rust parquet reader can parse them
    with chrono::DateTime::parse_from_rfc3339 without depending on Polars'
    datetime serialization specifics. EventsSchema enforces the RFC 3339 shape
    and `ends_at >= starts_at` (matching the DB CHECK constraint).
    """
    EventsSchema.write_parquet(load_events(), OUTPUT_DIR_PATH / "events.parquet")
