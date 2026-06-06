from pathlib import Path

import dataframely as dy
import polars as pl

from external.schemas._opening_hours_osm import assert_osm_parses
from external.schemas.opening_hours import OpeningHoursSchema

SOURCES_PATH = Path(__file__).parent.parent.parent / "sources"
OPENING_HOURS_CSV = SOURCES_PATH / "opening_hours.csv"


def load_opening_hours() -> dy.DataFrame[OpeningHoursSchema]:
    """
    Build the opening-hours frame from `data/sources/opening_hours.csv`.

    The CSV column names already match the schema. Empty optional fields
    (`valid_from`, `valid_until`, `service`) read back as null. After structural
    validation, every OSM string is parsed so an unparseable hand-authored
    schedule fails the build rather than shipping silently.
    """
    df = pl.read_csv(OPENING_HOURS_CSV, schema=OpeningHoursSchema.to_polars_schema())
    validated = OpeningHoursSchema.validate(df)
    for row in validated.iter_rows(named=True):
        assert_osm_parses(row["opening_hours"], entry_id=row["id"])
    return validated
