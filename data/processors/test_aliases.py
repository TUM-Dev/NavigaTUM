import json

import polars as pl
from pipeline_types import FlatRow

from processors.aliases import add_aliases, building_short_name_lookup
from processors.df_utils import unflatten_row


def _meta(rows: list[FlatRow]) -> pl.DataFrame:
    """Build the (id, type, short_name, parents) frame the lookup consumes."""
    return pl.DataFrame(rows, infer_schema_length=None)


def _as_dict(lookup: pl.DataFrame) -> dict[str, str]:
    """Collapse the lookup frame into a plain dict for ergonomic assertions."""
    return {row["id"]: row["building_short_name"] for row in lookup.to_dicts()}


def test_lookup_uses_own_short_name() -> None:
    """An entry with its own code-like short_name resolves to it without walking."""
    meta = _meta([{"id": "5204", "type": "building", "short_name": "UTG", "parents": ["root"]}])
    assert _as_dict(building_short_name_lookup(meta)) == {"5204": "UTG"}


def test_lookup_walks_up_to_joined_building() -> None:
    """The MW case: building 5510 has no short_name of its own, so it borrows MW from the joined_building."""
    meta = _meta(
        [
            {"id": "mw", "type": "joined_building", "short_name": "MW", "parents": ["root", "garching"]},
            {"id": "garching", "type": "campus", "short_name": None, "parents": ["root"]},
            {"id": "5510", "type": "building", "short_name": None, "parents": ["root", "garching", "mw"]},
        ],
    )
    assert _as_dict(building_short_name_lookup(meta))["5510"] == "MW"


def test_lookup_stops_at_geographic_ancestor() -> None:
    """Campus/area short_names must not leak into room aliases."""
    meta = _meta(
        [
            {"id": "garching", "type": "campus", "short_name": "Garching", "parents": ["root"]},
            {"id": "9999", "type": "building", "short_name": None, "parents": ["root", "garching"]},
        ],
    )
    assert "9999" not in _as_dict(building_short_name_lookup(meta))


def test_lookup_excludes_non_code_like_short_names() -> None:
    """Descriptive short_names would yield nonsensical "Mathe/Info (MI)0001" aliases."""
    meta = _meta([{"id": "mi", "type": "joined_building", "short_name": "Mathe/Info (MI)", "parents": ["root"]}])
    assert "mi" not in _as_dict(building_short_name_lookup(meta))


def _lookup_df(mapping: dict[str, str]) -> pl.DataFrame:
    """Build the lookup frame ``add_aliases`` expects from a compact dict literal."""
    return pl.DataFrame(
        {"id": list(mapping), "building_short_name": list(mapping.values())},
        schema={"id": pl.Utf8, "building_short_name": pl.Utf8},
    )


def _aliases_for(arch: str | None, lookup: dict[str, str], entry_type: str = "room", _id: str = "x") -> list[str]:
    """Run ``add_aliases`` on a single synthetic row and return the resulting alias list."""
    td = json.dumps({"arch_name": arch}) if arch is not None else None
    # tumonline_data_json is Utf8 in the pipeline (ensure_columns); pin it so an all-null
    # test column does not infer Null dtype and break json_path_match.
    lf = pl.DataFrame(
        {"id": [_id], "type": [entry_type], "tumonline_data_json": [td]},
        schema={"id": pl.Utf8, "type": pl.Utf8, "tumonline_data_json": pl.Utf8},
    ).lazy()
    out = add_aliases(lf, _lookup_df(lookup)).collect()
    aliases: list[str] = out["aliases"][0].to_list()
    return aliases


def test_add_aliases_derives_friendly_form() -> None:
    """A "<number>@<building_id>" arch_name gains the "<short_name><number>" alias next to the raw form."""
    assert _aliases_for("0001@5510", {"5510": "MW"}) == ["0001@5510", "MW0001"]


def test_add_aliases_keeps_raw_when_building_has_no_short_name() -> None:
    """No backfill: an unknown building short_name leaves only the raw form."""
    assert _aliases_for("0001@5510", {}) == ["0001@5510"]


def test_add_aliases_building_gets_no_friendly_form() -> None:
    """Buildings carry "@<id>" with an empty number, so no friendly alias is derived."""
    assert _aliases_for(None, {"5510": "MW"}, entry_type="building", _id="5510") == ["@5510"]


def test_add_aliases_handles_arch_name_without_at() -> None:
    """Malformed arch_names with no "@" keep the raw form and derive nothing."""
    assert _aliases_for("noat", {"5510": "MW"}) == ["noat"]


def test_add_aliases_empty_when_no_arch_name() -> None:
    """Entries without an arch_name produce an empty alias list."""
    assert _aliases_for(None, {}) == []


def test_unflatten_always_emits_aliases() -> None:
    """
    The /locations/:id API requires `aliases` on every entry, so an empty list must survive the round-trip.

    Regression: an empty list is falsy, so a truthiness guard silently dropped the field for
    alias-less entries (e.g. the `mi` joined_building), and the server rejected the detail as
    `missing field 'aliases'`.
    """
    assert unflatten_row({"id": "mi", "type": "joined_building", "aliases": []})["aliases"] == []
    assert unflatten_row({"id": "mi", "type": "joined_building"})["aliases"] == []
    assert unflatten_row({"id": "x", "type": "room", "aliases": ["0001@5510", "MW0001"]})["aliases"] == [
        "0001@5510",
        "MW0001",
    ]
