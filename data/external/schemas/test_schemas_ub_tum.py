import dataframely as dy
import opening_hours
import polars as pl
import pytest

from external.loaders.ub_tum import load_ub_tum
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.ub_tum import UbTumSchema
from external.scrapers.ub_tum import parse_branch_page

_DAYS = ("Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday")


def _paragraph(label: str, slots: dict[str, str]) -> str:
    """
    Build the Drupal `office_hours` markup for one paragraph block.

    `slots` maps `_DAYS` entries to a slot string (e.g. `8:00-24:00` or `Closed`);
    omitted weekdays render with no row, which the parser treats as closed.
    """
    rows = "".join(
        (
            f'<div class="office-hours__item">'
            f'<span class="office-hours__item-label">{day}: </span>'
            f'<span class="office-hours__item-slots">{slots[day]}</span>'
            f"</div>"
        )
        for day in _DAYS
        if day in slots
    )
    return (
        f'<div class="paragraph paragraph--type--oeffnungszeiten">'
        f'<div class="field field--name-field-zeiten field--type-office-hours">'
        f'<div class="field__label">{label}</div>'
        f'<div class="field__items"><div class="field__item"><div class="office-hours">{rows}</div></div></div>'
        f"</div></div>"
    )


# A captured slice of the live `branch-library-mathematics-informatics` page: one
# `paragraph--type--oeffnungszeiten` block, label `Zeiten`, seven daily slot rows.
_MATH_INFO_HTML = _paragraph(
    "Zeiten",
    dict.fromkeys(("Monday", "Tuesday", "Wednesday", "Thursday", "Friday"), "8:00-24:00")
    | dict.fromkeys(("Saturday", "Sunday"), "10:00-20:00"),
)

# A closed-weekend page (the shape every branch except math-informatics, weihenstephan,
# and main-campus currently publishes), used to confirm closed days are dropped from
# the OSM string rather than emitted as `Sa-Su Closed`.
_MEDICINE_HTML = _paragraph(
    "Zeiten",
    dict.fromkeys(("Monday", "Tuesday", "Wednesday", "Thursday", "Friday"), "8:00-21:00")
    | dict.fromkeys(("Saturday", "Sunday"), "Closed"),
)

# A synthesised two-paragraph fixture: the shape `ub.tum.de` is expected to publish
# once it starts distinguishing lecture period from semester break (per #3050). The
# real branch pages currently publish a single year-round paragraph, so this fixture
# guards the future state and the macro plumbing it exercises.
_LECTURE_BREAK_HTML = _paragraph(
    "Lecture period",
    dict.fromkeys(("Monday", "Tuesday", "Wednesday", "Thursday", "Friday"), "8:00-24:00")
    | dict.fromkeys(("Saturday", "Sunday"), "10:00-20:00"),
) + _paragraph(
    "Semester break",
    dict.fromkeys(("Monday", "Tuesday", "Wednesday", "Thursday", "Friday"), "9:00-20:00")
    | dict.fromkeys(("Saturday", "Sunday"), "10:00-16:00"),
)

# A synthesised service-variant fixture: a separate paragraph whose label is not a
# season key, so the parser must turn it into a per-rule trailing OSM comment that
# the renderer can group on. Mirrors the medicine-library pickup case in #3050.
_SERVICE_VARIANT_HTML = _paragraph(
    "Zeiten",
    dict.fromkeys(("Monday", "Tuesday", "Wednesday", "Thursday", "Friday"), "8:00-21:00"),
) + _paragraph(
    "Pickup of preordered books",
    dict.fromkeys(("Monday", "Tuesday", "Wednesday", "Thursday", "Friday"), "9:00-20:00"),
)


def _valid_row() -> dict[str, list[object]]:
    """Build a single valid UB-TUM row."""
    return {
        "branch_id": ["mathematics-informatics"],
        "name": ["Mathematics & Informatics"],
        "opening_hours": ["Mo-Fr 08:00-24:00; Sa-Su 10:00-20:00"],
        "last_update": ["2026-06-08"],
        "source_url": ["https://www.ub.tum.de/en/branch-library-mathematics-informatics"],
    }


def _row_with(**overrides: object) -> pl.DataFrame:
    """Build a one-row frame from the valid baseline, overriding named columns."""
    row = _valid_row()
    for key, value in overrides.items():
        row[key] = [value]
    return pl.DataFrame(row, schema=UbTumSchema.to_polars_schema())


def test_committed_ub_tum_csv_satisfies_schema() -> None:
    """The cached `ub_tum.csv` must satisfy `UbTumSchema` (drift gate)."""
    assert_satisfies_schema(UbTumSchema, load_ub_tum())


def test_parse_branch_page_collapses_year_round_block() -> None:
    """A single `Zeiten` paragraph collapses into plain OSM with consecutive days grouped."""
    parsed = parse_branch_page(
        _MATH_INFO_HTML,
        source_url="https://www.ub.tum.de/en/branch-library-mathematics-informatics",
        branch_id="mathematics-informatics",
        name="Mathematics & Informatics",
    )
    assert parsed.opening_hours == "Mo-Fr 08:00-24:00; Sa-Su 10:00-20:00"
    assert opening_hours.validate(parsed.opening_hours)


def test_parse_branch_page_drops_closed_days() -> None:
    """`Closed` slot cells must not appear in the OSM output."""
    parsed = parse_branch_page(
        _MEDICINE_HTML,
        source_url="https://www.ub.tum.de/en/branch-library-medicine",
        branch_id="medicine",
        name="Medicine",
    )
    assert parsed.opening_hours == "Mo-Fr 08:00-21:00"
    assert opening_hours.validate(parsed.opening_hours)


def test_parse_branch_page_maps_lecture_and_break_paragraphs_to_macros() -> None:
    """`Lecture period` / `Semester break` labels prefix each rule with the matching macro."""
    parsed = parse_branch_page(
        _LECTURE_BREAK_HTML,
        source_url="https://www.ub.tum.de/en/branch-library-mathematics-informatics",
        branch_id="mathematics-informatics",
        name="Mathematics & Informatics",
    )
    assert parsed.opening_hours == (
        "lecture: Mo-Fr 08:00-24:00; lecture: Sa-Su 10:00-20:00; break: Mo-Fr 09:00-20:00; break: Sa-Su 10:00-16:00"
    )


def test_parse_branch_page_turns_unknown_label_into_trailing_comment() -> None:
    """An unknown paragraph label becomes a per-rule trailing comment (service variant)."""
    parsed = parse_branch_page(
        _SERVICE_VARIANT_HTML,
        source_url="https://www.ub.tum.de/en/branch-library-medicine",
        branch_id="medicine",
        name="Medicine",
    )
    assert parsed.opening_hours == 'Mo-Fr 08:00-21:00; Mo-Fr 09:00-20:00 "Pickup of preordered books"'
    assert opening_hours.validate(parsed.opening_hours)


def test_parse_branch_page_rejects_a_page_without_office_hours() -> None:
    """A page that publishes no opening-hours paragraph must raise rather than emit empty hours."""
    with pytest.raises(ValueError, match="no opening-hours paragraph"):
        parse_branch_page(
            "<html><body>nothing here</body></html>",
            source_url="https://www.ub.tum.de/en/branch-library-broken",
            branch_id="broken",
            name="Broken",
        )


def test_parse_branch_page_rejects_a_page_with_only_closed_days() -> None:
    """A paragraph in which every slot is `Closed` must raise rather than emit an empty string."""
    closed_only = _paragraph("Zeiten", dict.fromkeys(_DAYS, "Closed"))
    with pytest.raises(ValueError, match="no opening-hours rules"):
        parse_branch_page(
            closed_only,
            source_url="https://www.ub.tum.de/en/branch-library-medicine",
            branch_id="medicine",
            name="Medicine",
        )


def test_ub_tum_schema_accepts_minimal_valid_row() -> None:
    """A row matching every rule must validate cleanly (positive control)."""
    UbTumSchema.validate(_row_with())


def test_ub_tum_schema_accepts_macro_form() -> None:
    """`lecture:`/`break:` macros are allowed; `merge_opening_hours` expands them later."""
    UbTumSchema.validate(_row_with(opening_hours="lecture: Mo-Fr 08:00-20:00"))


def test_ub_tum_schema_rejects_duplicate_branch() -> None:
    """`UbTumSchema` must reject a duplicated `branch_id`."""
    duplicated = pl.DataFrame(
        {
            "branch_id": ["medicine", "medicine"],
            "name": ["Medicine", "Medicine"],
            "opening_hours": ["Mo-Fr 08:00-21:00", "Mo-Fr 08:00-21:00"],
            "last_update": ["2026-06-08", "2026-06-08"],
            "source_url": ["https://x.tld", "https://x.tld"],
        },
        schema=UbTumSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        UbTumSchema.validate(duplicated)


def test_ub_tum_schema_rejects_empty_opening_hours() -> None:
    """An empty `opening_hours` string must be rejected."""
    with pytest.raises(dy.exc.ValidationError):
        UbTumSchema.validate(_row_with(opening_hours=""))


@pytest.mark.parametrize("url", ["www.ub.tum.de", "ftp://ub.tum.de", "/relative/path", ""])
def test_ub_tum_schema_rejects_non_http_source_url(url: str) -> None:
    """`source_url` must be an absolute http(s) URL."""
    with pytest.raises(dy.exc.ValidationError):
        UbTumSchema.validate(_row_with(source_url=url))


@pytest.mark.parametrize("bad_date", ["2026/06/08", "08-06-2026", "2026-6-8", "not-a-date"])
def test_ub_tum_schema_rejects_non_iso_last_update(bad_date: str) -> None:
    """`last_update` must be a `YYYY-MM-DD` date."""
    with pytest.raises(dy.exc.ValidationError):
        UbTumSchema.validate(_row_with(last_update=bad_date))


def test_ub_tum_schema_rejects_missing_column() -> None:
    """`UbTumSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"branch_id": ["medicine"]})
    with pytest.raises(dy.exc.SchemaError):
        UbTumSchema.validate(incomplete)
