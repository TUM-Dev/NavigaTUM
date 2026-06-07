from itertools import pairwise

import dataframely as dy
import polars as pl
import pytest
from processors.semester_block_expander import Semester

from external.loaders.semesters import load_semester
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.semesters import SemesterSchema


def _valid_row() -> dict[str, list[object]]:
    """Build a single valid semester row."""
    return {
        "key": ["2025S"],
        "start": ["2025-04-01"],
        "lectures_from": ["2025-04-22"],
        "lectures_until": ["2025-08-02"],
        "end": ["2025-09-30"],
    }


def _row_with(**overrides: object) -> pl.DataFrame:
    """Build a one-row frame from the valid baseline, overriding named columns."""
    row = _valid_row()
    for key, value in overrides.items():
        row[key] = [value]
    return pl.DataFrame(row, schema=SemesterSchema.to_polars_schema())


def test_committed_semesters_csv_satisfies_schema() -> None:
    """The committed `semesters.csv` must satisfy `SemesterSchema` (drift gate)."""
    assert_satisfies_schema(SemesterSchema, load_semester())


def test_committed_semesters_are_chronological_and_disjoint() -> None:
    """Successive semesters must not overlap, so a given day maps to at most one semester."""
    semesters = sorted(
        (Semester.from_row(row) for row in load_semester().iter_rows(named=True)),
        key=lambda semester: semester.start,
    )
    for earlier, later in pairwise(semesters):
        assert earlier.end < later.start, f"{earlier.key} overlaps {later.key}"


def test_semester_schema_accepts_minimal_valid_row() -> None:
    """A row matching every rule must validate cleanly (positive control)."""
    SemesterSchema.validate(_row_with())


def test_semester_schema_rejects_inverted_calendar_range() -> None:
    """A semester ending before it starts must be rejected."""
    with pytest.raises(dy.exc.ValidationError):
        SemesterSchema.validate(_row_with(start="2025-09-30", end="2025-04-01"))


def test_semester_schema_rejects_inverted_lecture_range() -> None:
    """Lectures ending before they start must be rejected."""
    with pytest.raises(dy.exc.ValidationError):
        SemesterSchema.validate(_row_with(lectures_from="2025-08-02", lectures_until="2025-04-22"))


@pytest.mark.parametrize(
    ("lectures_from", "lectures_until"),
    [("2025-03-01", "2025-08-02"), ("2025-04-22", "2025-10-15")],
)
def test_semester_schema_rejects_lectures_outside_calendar(lectures_from: str, lectures_until: str) -> None:
    """The lecture period must fall inside the semester calendar span."""
    with pytest.raises(dy.exc.ValidationError):
        SemesterSchema.validate(_row_with(lectures_from=lectures_from, lectures_until=lectures_until))
