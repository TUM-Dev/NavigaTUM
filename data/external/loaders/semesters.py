from pathlib import Path

import dataframely as dy
import polars as pl
from processors.semester_block_expander import Semester

from external.schemas.semesters import SemesterSchema

SOURCES_PATH = Path(__file__).parent.parent.parent / "sources"
SEMESTERS_CSV = SOURCES_PATH / "semesters.csv"


def load_semester_frame() -> dy.DataFrame[SemesterSchema]:
    """Load and validate `data/sources/semesters.csv`."""
    df = pl.read_csv(SEMESTERS_CSV, schema=SemesterSchema.to_polars_schema())
    return SemesterSchema.validate(df)


def load_semesters() -> list[Semester]:
    """Return the committed semester fixture as `Semester` values for the macro expander."""
    return [Semester.from_row(row) for row in load_semester_frame().iter_rows(named=True)]
