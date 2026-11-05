from pathlib import Path

import yaml

from processors.patch import apply_roomcode_patch


def test_4277_ground_floor_architect_names_are_corrected() -> None:
    """Verify corrected architect names and preserve rooms without a matching patch."""
    sources_path = Path(__file__).parent.parent / "sources"
    with (sources_path / "15_patches-rooms_tumonline.yaml").open(encoding="utf-8") as file:
        patches = yaml.safe_load(file)["patches"]

    expected = {
        "4277.EG.001": ("1.2.EG1, Sammlungsraum", "1.2.EG1@4277"),
        "4277.EG.003": ("1.2.EG5, Büro", "1.2.EG5@4277"),
        "4277.EG.004": ("1.2.EG7, Büro", "1.2.EG7@4277"),
        "4277.EG.005": ("1.2.EG9, Sozial-/und Aufenthaltsraum", "1.2.EG9@4277"),
        "4277.EG.006": ("1.2.EG.1, Treppe", "1.2.EG.1@4277"),
        "4277.EG.007": ("1.2.EG11, Büro Mitarbeiter", "1.2.EG11@4277"),
        "4277.EG.008": ("1.2.EG13, Büro Mitarbeiter", "1.2.EG13@4277"),
        "4277.EG.011": ("1.2.EG19, Büro Mitarbeiter", "1.2.EG19@4277"),
        "4277.EG.012": ("1.2.EG21, Büro/Besprechungs- & Übungsraum", "1.2.EG21@4277"),
        "4277.EG.013": ("1.2.EG, Flur", "1.2.EG@4277"),
    }
    rooms = {room_code: {"alt_name": None, "arch_name": None} for room_code in [*expected, "4277.EG.002"]}

    apply_roomcode_patch(rooms, patches)

    for room_code, (alt_name, arch_name) in expected.items():
        assert rooms[room_code] == {"alt_name": alt_name, "arch_name": arch_name, "patched": True}
    assert rooms["4277.EG.002"] == {"alt_name": None, "arch_name": None}
