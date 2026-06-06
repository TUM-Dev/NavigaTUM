"""
Authoritative OSM `opening_hours` syntax gate for the compile step.

The `OpeningHoursSchema` rules only cover structure (non-empty, no macros, valid
dates and URL); they cannot decide whether the string is a grammatically valid OSM
`opening_hours` expression. This wraps the `opening-hours` Rust parser so the build
fails loudly on an unparseable hand-authored schedule, rather than shipping a string
the webclient parser would later choke on.
"""

import opening_hours


def assert_osm_parses(osm: str, *, entry_id: str | None = None) -> None:
    """
    Validate that ``osm`` parses as an OSM ``opening_hours`` expression.

    :raises ValueError: if the string is not a valid OSM ``opening_hours`` expression.
    """
    if not opening_hours.validate(osm):
        where = f" for entry {entry_id!r}" if entry_id else ""
        raise ValueError(f"opening_hours{where} does not parse as OSM: {osm!r}")
