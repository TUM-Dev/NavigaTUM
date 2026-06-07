from typing import Any

from utils import TranslatableStr

from processors.export import extract_exported_item


def _data() -> dict[str, dict[str, Any]]:
    """Build a minimal ancestor chain root -> garching -> mi, keyed by id as the pipeline keys it."""
    return {
        "root": {"id": "root", "type": "root", "name": TranslatableStr("Standorte", "Sites"), "parents": []},
        "garching": {
            "id": "garching",
            "type": "site",
            "name": TranslatableStr("Garching", "Garching"),
            "parents": ["root"],
        },
        "mi": {
            "id": "mi",
            "type": "building",
            "name": TranslatableStr("MI", "MI"),
            "parents": ["root", "garching"],
        },
    }


def test_parent_types_parallel_array_carries_each_parents_type():
    """`parent_types` mirrors `parents`/`parent_names` so the client can build /{type}/{id} links."""
    data = _data()
    result = extract_exported_item(data, data["mi"])

    assert result["parent_types"] == ["root", "site"]


def test_parents_and_parent_names_are_unchanged():
    """The additive `parent_types` field leaves the existing parallel arrays intact (rollout-safe)."""
    data = _data()
    result = extract_exported_item(data, data["mi"])

    assert result["parents"] == ["root", "garching"]
    assert result["parent_names"] == [TranslatableStr("Standorte", "Sites"), TranslatableStr("Garching", "Garching")]


def test_root_parent_type_is_synthesised_without_a_data_entry():
    """`root` has no real data entry, so its type is supplied directly rather than looked up."""
    data = _data()
    result = extract_exported_item(data, data["garching"])

    assert result["parent_types"] == ["root"]
