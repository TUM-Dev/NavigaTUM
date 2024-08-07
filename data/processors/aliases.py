from typing import Any



def extract_arch_name(entry: dict) -> str | None:
    """Extract the arch name from the entry"""
    if entry["type"] == "building":
        return f"@{entry['id']}"
    return entry.get("tumonline_data", {}).get("arch_name", None)

def add_aliases(data: dict[str, dict[str, Any]]) -> None:
    """Add coordinates to all entries and check for issues"""
    
    for _id, entry in data.items():
        entry["aliases"] = []
        if arch_name := extract_arch_name(entry):
            entry["arch_name"] = arch_name,
            entry["aliases"].append(arch_name)