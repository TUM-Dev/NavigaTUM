import dataframely as dy
import polars as pl


class SemesterSchema(dy.Schema):
    """
    One academic semester, used to expand `lecture:`/`break:` opening-hours macros.

    Rows come from the committed `data/sources/semesters.csv`. `key` (e.g. `2025S`)
    is the primary key. Dates are real `Date` columns (never client-facing, so no
    locale concern) because the expander does `date` arithmetic on them.
    """

    key = dy.String(nullable=False, primary_key=True)
    start = dy.Date(nullable=False)
    lectures_from = dy.Date(nullable=False)
    lectures_until = dy.Date(nullable=False)
    end = dy.Date(nullable=False)

    @dy.rule()
    def calendar_range_ordered(cls) -> pl.Expr:
        """Reject a semester that ends before it starts."""
        return pl.col("end") >= pl.col("start")

    @dy.rule()
    def lecture_range_ordered(cls) -> pl.Expr:
        """Reject lectures that end before they start."""
        return pl.col("lectures_until") >= pl.col("lectures_from")

    @dy.rule()
    def lectures_within_calendar(cls) -> pl.Expr:
        """Require the lecture period to fall inside the semester calendar span."""
        return (pl.col("lectures_from") >= pl.col("start")) & (pl.col("lectures_until") <= pl.col("end"))
