from pathlib import Path

import dataframely as dy
import polars as pl

from external.schemas.opening_hours import OpeningHoursSchema

SOURCES_PATH = Path(__file__).parent.parent.parent / "sources"
OPENING_HOURS_CSV = SOURCES_PATH / "opening_hours.csv"


def load_opening_hours() -> dy.DataFrame[OpeningHoursSchema]:
    """
    Load and validate `data/sources/opening_hours.csv`.

    Empty optional fields read back as null. OSM-string syntax is gated by the
    test suite, not at runtime.
    """
    df = pl.read_csv(OPENING_HOURS_CSV, schema=OpeningHoursSchema.to_polars_schema())
    return OpeningHoursSchema.validate(df)
