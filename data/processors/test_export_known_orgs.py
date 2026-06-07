import orjson
from external.loaders.tumonline_orgs import load_tumonline_orgs

from processors.export import OUTPUT_DIR_PATH, export_known_orgs


def test_export_known_orgs_writes_picker_fields_sorted_by_name() -> None:
    """`known_orgs.json` lists every org with exactly the picker fields, sorted by `name_de`."""
    export_known_orgs()

    raw = (OUTPUT_DIR_PATH / "known_orgs.json").read_bytes()
    assert raw.endswith(b"\n"), "must mirror known_usages.json's trailing newline"
    orgs = orjson.loads(raw)

    expected_count = load_tumonline_orgs().height
    assert len(orgs) == expected_count, "one row per validated TUMonline org"

    for org in orgs:
        assert list(org.keys()) == ["org_id", "code", "name_de", "name_en"], "stable field set and order for the frontend contract"
        assert isinstance(org["org_id"], int)
        assert org["org_id"] > 0, "org_id is the positive key submitted as organising_org_id"
        assert org["code"].strip(), "code is the non-empty TUMonline org code"
        assert org["name_de"], "German name is required for display and filtering"
        assert org["name_en"], "English name is required for display and filtering"

    names = [org["name_de"] for org in orgs]
    assert names == sorted(names), "deterministic default order for the combobox"
