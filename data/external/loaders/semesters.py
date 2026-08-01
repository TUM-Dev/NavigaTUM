from pathlib import Path

import dataframely as dy
import polars as pl

from external.schemas.semesters import SemesterSchema

SOURCES_PATH = Path(__file__).parent.parent.parent / "sources"
SEMESTERS_CSV = SOURCES_PATH / "semesters.csv"


def load_semester() -> dy.DataFrame[SemesterSchema]:
    """Load and validate `data/sources/semesters.csv`."""
    df = pl.read_csv(SEMESTERS_CSV, schema=pl.Schema(SemesterSchema))
    return SemesterSchema.validate(df)
