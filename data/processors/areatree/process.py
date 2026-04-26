import logging
import re
from collections.abc import Iterator
from pathlib import Path

import polars as pl

from processors.areatree import models
from processors.df_utils import to_json_or_none

_logger = logging.getLogger(__name__)

AREATREE_FILE = Path(__file__).parent / "config.areatree"

# Trailing parenthetical that looks like a short name / building code rather than a clarification.
# Matches things like "(N1)", "(Z11)", "(SW3)", "(BC1)", "(Bau 501)", "(BL. G)", "(BT07)",
# "(CH1)", "(CH 1)", "(MW25)", "(WSI)", "(CRC)", "(IAS)", "(KBA)", "(MAK/BUA)", "(SG 16)".
# Excludes longer descriptors like "(GRS-Altbau)", "(orange)", "(hellgrün)" by limiting length
# and requiring an uppercase-dominant token shape.
_EMBEDDED_SHORT_NAME_RE = re.compile(
    r"\s*\((?:"
    r"Bau\s+\d+"  # Bau 501
    r"|BL\.?\s*[A-Z]"  # BL. G, BL F
    r"|BT\d+"  # BT07
    r"|[A-Z]{1,3}\s?\d+[A-Z]?"  # N1, Z11, SW3, BC1, CH 1, MW25, SG 16
    r"|[A-Z]{2,6}(?:[-/][A-Z]{2,6})?"  # WSI, CRC, MAK/BUA
    r")\)\s*$"
)
# Leading short-name acronym pattern. Requires multiple uppercase letters in the leading token
# (so it catches "BNMRZ", "HEZ", "LWF", "LfL", "HfP", "iGZW") but skips ordinary capitalised
# words like "Campus", "Mensa", "Halle", "Munich".
_LEADING_ACRONYM_RE = re.compile(
    r"^("
    r"[A-Z]{2,6}"  # all caps: BNMRZ, HEZ, LWF, KFZ, HSWT, BLQ
    r"|[a-z][A-Z][A-Za-z]{1,4}"  # leading-lower: iGZW
    r"|[A-Z][a-z][A-Z][A-Za-z]{0,3}"  # HfP, LfL
    r")\s+[A-ZÄÖÜ][a-zäöüß]"
)
# Institutional-brand prefixes that are NOT short names. Skip the warning entirely for these.
_INSTITUTIONAL_BRANDS: frozenset[str] = frozenset({"TUM", "LMU"})
# TUMonline operator/location markers that occasionally bleed into areatree names. They are not
# real short names — suggest dropping them.
_TUMONLINE_NOISE_MARKERS: frozenset[str] = frozenset({"AM", "NR", "SZ", "GP", "GM"})
# Trailing parentheticals that look like building/identifier codes. When matched these should
# become a ``visible_id`` rather than a short_name.
_CODE_LIKE_RE = re.compile(
    r"^(?:"
    r"Bau\s+\d+"  # Bau 501
    r"|BL\.?\s*[A-Z]"  # BL. G
    r"|BT\d+"  # BT07
    r"|CH\s*\d+|MW\s*\d+|SG\s*\d+|PG\s*\d+"  # CH 1, MW25, SG 16, PG 18
    r"|[A-Z]{1,3}\s?\d+[A-Z]?"  # N1, Z11, SW3, BC1
    r")$"
)


def read_areatree() -> pl.DataFrame:
    """Read the areatree file and the basic data, gained from the areatree"""
    parent_stack: list[str] = []
    last_element: str = ""
    rows = []
    for line in _areatree_lines():
        indent = len(line) - len(line.lstrip(" "))
        if indent % 2 != 0:
            raise RuntimeError(f"Indentation not multiple of 2: '{line}'")
        if (indent // 2) > len(parent_stack):
            parent_stack.append(last_element)
        elif (indent // 2) < len(parent_stack):
            parent_stack = parent_stack[: indent // 2]

        building_data = _parse_areatree_line(line, parent_stack[:])
        row = {
            "id": building_data["id"],
            "type": building_data["type"],
            "name": building_data["name"],
            "name_de": building_data["name"],
            "name_en": building_data["name"],
            "parents": building_data["parents"],
        }
        if "b_prefix" in building_data:
            bp = building_data["b_prefix"]
            if isinstance(bp, list):
                row["b_prefix_list"] = bp
            else:
                row["b_prefix"] = bp
        if "visible_id" in building_data:
            row["visible_id"] = building_data["visible_id"]
        if "short_name" in building_data:
            row["short_name"] = building_data["short_name"]
            row["short_name_de"] = building_data["short_name"]
            row["short_name_en"] = building_data["short_name"]
        if "data_quality" in building_data:
            row["data_quality_json"] = to_json_or_none(building_data["data_quality"])  # type: ignore[assignment]
        rows.append(row)
        last_element = building_data["id"]
    return pl.DataFrame(rows, infer_schema_length=None)


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
    raw_name_parts = raw_names.split("|")
    names = _extract_names(raw_name_parts)
    id_and_type = _extract_id_and_type(internal_id, building_data.get("b_prefix"))
    visible_id_raw = id_and_type.get("visible_id")
    _warn_if_embedded_short_name(
        names["name"],
        has_explicit_short_name=len(raw_name_parts) > 1,
        building_ids=building_ids,
        entry_id=id_and_type["id"],
        visible_id=visible_id_raw if isinstance(visible_id_raw, str) else None,
    )
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
            _logger.warning(f"'{names[1]}' is very long for a short name (>20 chars)")

        building_data["short_name"] = names[1]
    elif len(names) > 2:
        raise RuntimeError(f"Too many names: {names}")
    return building_data


def _normalize_id(value: str) -> str:
    """Lowercase + strip whitespace/punctuation so 'BL. G' matches 'bl-g'."""
    return re.sub(r"[\s\-_./]", "", value).lower()


def _format_line(building_ids: str, name: str, entry_id: str, visible_id: str | None) -> str:
    """Reconstruct a full areatree line `<building_ids>:<name>:<id>[,<visible_id>]`."""
    internal = entry_id if not visible_id else f"{entry_id},{visible_id}"
    return f"{building_ids}:{name}:{internal}"


def _warn_if_embedded_short_name(
    name: str,
    has_explicit_short_name: bool,
    building_ids: str,
    entry_id: str,
    visible_id: str | None,
) -> None:
    """
    Flag names that bake a short-name into the long name.

    Four cases are distinguished:

    1. **TUMonline noise** (``(AM)``/``(NR)``/``(SZ)``): suggest dropping outright.
    2. **Duplicates the visible_id**: parenthetical equals the existing visible_id; drop it.
    3. **Code-like parenthetical** without a visible_id: promote to ``visible_id``.
    4. **Acronym short-name**: migrate to the explicit ``long name|short name`` syntax.

    Leading-acronym patterns are also detected, with hardcoded skips for institutional brands
    (``TUM``, ``LMU``) where the prefix is just branding rather than a real short name.
    """
    if has_explicit_short_name:
        return
    visible_norm = _normalize_id(visible_id) if visible_id else ""

    trailing = _EMBEDDED_SHORT_NAME_RE.search(name)
    if trailing:
        short = trailing.group(0).strip().strip("()").strip()
        long_name = _EMBEDDED_SHORT_NAME_RE.sub("", name).rstrip(" ,;")
        short_norm = _normalize_id(short)

        # Case 1: TUMonline operator/location marker — just noise.
        if short.upper() in _TUMONLINE_NOISE_MARKERS:
            fixed = _format_line(building_ids, long_name, entry_id, visible_id)
            _logger.warning(
                f"'{entry_id}': name '{name}' contains TUMonline noise marker '({short})'. "
                f"Drop it — line should be '{fixed}'"
            )
            return

        # Case 2: parenthetical duplicates the existing visible_id.
        if visible_norm and short_norm == visible_norm:
            fixed = _format_line(building_ids, long_name, entry_id, visible_id)
            _logger.warning(
                f"'{entry_id}': name '{name}' duplicates the visible_id '{visible_id}'. "
                f"Drop the trailing '({short})' — line should be '{fixed}'"
            )
            return

        # Case 3: looks like a code (Bau 501, BL. G, BT07, N1, ...) — make it the visible_id.
        if not visible_id and _CODE_LIKE_RE.match(short):
            new_visible = short_norm
            fixed = _format_line(building_ids, long_name, entry_id, new_visible)
            _logger.warning(
                f"'{entry_id}': name '{name}' embeds the code '({short})'. "
                f"Promote it to a visible_id — line should be '{fixed}'"
            )
            return

        # Case 4: acronym short_name — use the |-syntax.
        fixed_name = f"{long_name}|{short}"
        fixed = _format_line(building_ids, fixed_name, entry_id, visible_id)
        _logger.warning(
            f"'{entry_id}': name '{name}' embeds the short name '{short}'. "
            f"Use the '|'-syntax — line should be '{fixed}'"
        )
        return

    leading = _LEADING_ACRONYM_RE.match(name)
    if leading:
        short = leading.group(1)
        # Case 1 (leading): institutional brand — meaningless as short_name, skip silently.
        if short in _INSTITUTIONAL_BRANDS:
            return

        long_name = name[len(short) :].lstrip(" ,;-")
        short_norm = _normalize_id(short)

        # Case 2 (leading): duplicates visible_id.
        if visible_norm and short_norm == visible_norm:
            fixed = _format_line(building_ids, long_name, entry_id, visible_id)
            _logger.warning(
                f"'{entry_id}': name '{name}' duplicates the visible_id '{visible_id}'. "
                f"Drop the leading '{short}' — line should be '{fixed}'"
            )
            return

        # Case 4 (leading): acronym short_name.
        fixed_name = f"{long_name}|{short}"
        fixed = _format_line(building_ids, fixed_name, entry_id, visible_id)
        _logger.warning(
            f"'{entry_id}': name '{name}' embeds the short name '{short}'. "
            f"Use the '|'-syntax — line should be '{fixed}'"
        )
