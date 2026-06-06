import dataframely as dy
import opening_hours
import polars as pl
import pytest

from external.loaders.studierendenwerk import load_studierendenwerk
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.studierendenwerk import StudierendenwerkSchema
from external.scrapers.studierendenwerk import open_hours_to_osm

# A captured slice of the eat-api `canteens.json` `open_hours` shape, covering the cases the
# converter must get right: a uniform Mo-Fr week, a week whose Friday differs, and a week with
# three distinct day groups. Kept as fixtures so the converter is verified without a network call.
_CAPTURED_OPEN_HOURS: dict[str, dict[str, dict[str, str]]] = {
    "mensa-arcisstr": {day: {"start": "11:00", "end": "14:00"} for day in ("mon", "tue", "wed", "thu", "fri")},
    "mensa-leopoldstr": {
        "mon": {"start": "11:00", "end": "14:30"},
        "tue": {"start": "11:00", "end": "14:30"},
        "wed": {"start": "11:00", "end": "14:30"},
        "thu": {"start": "11:00", "end": "14:30"},
        "fri": {"start": "11:00", "end": "14:00"},
    },
    "stucafe-pasing": {
        "mon": {"start": "07:45", "end": "16:15"},
        "tue": {"start": "07:45", "end": "16:15"},
        "wed": {"start": "07:45", "end": "16:00"},
        "thu": {"start": "07:45", "end": "16:00"},
        "fri": {"start": "07:45", "end": "14:30"},
    },
}


def _valid_row() -> dict[str, list[object]]:
    """Build a single valid canteen row."""
    return {
        "canteen_id": ["mensa-garching"],
        "name": ["Mensa Garching"],
        "opening_hours": ["Mo-Fr 10:45-14:15"],
        "last_update": ["2026-06-05"],
        "source_url": ["https://tum-dev.github.io/eat-api/#!/de/mensa-garching"],
    }


def _row_with(**overrides: object) -> pl.DataFrame:
    """Build a one-row frame from the valid baseline, overriding named columns."""
    row = _valid_row()
    for key, value in overrides.items():
        row[key] = [value]
    return pl.DataFrame(row, schema=StudierendenwerkSchema.to_polars_schema())


def test_committed_studierendenwerk_csv_satisfies_schema() -> None:
    """The committed `studierendenwerk.csv` must satisfy `StudierendenwerkSchema` (drift gate)."""
    assert_satisfies_schema(StudierendenwerkSchema, load_studierendenwerk())


def test_committed_studierendenwerk_strings_parse_as_osm() -> None:
    """Every committed OSM string must parse; the loader does not check this at runtime."""
    for row in load_studierendenwerk().iter_rows(named=True):
        assert opening_hours.validate(row["opening_hours"]), (
            f"opening_hours for canteen {row['canteen_id']!r} does not parse: {row['opening_hours']!r}"
        )


@pytest.mark.parametrize(
    ("canteen_id", "expected"),
    [
        ("mensa-arcisstr", "Mo-Fr 11:00-14:00"),
        ("mensa-leopoldstr", "Mo-Th 11:00-14:30; Fr 11:00-14:00"),
        ("stucafe-pasing", "Mo-Tu 07:45-16:15; We-Th 07:45-16:00; Fr 07:45-14:30"),
    ],
)
def test_open_hours_to_osm_groups_consecutive_days(canteen_id: str, expected: str) -> None:
    """Captured feed fixtures must convert to the expected (and parseable) OSM string."""
    osm = open_hours_to_osm(_CAPTURED_OPEN_HOURS[canteen_id])
    assert osm == expected
    assert opening_hours.validate(osm)


def test_open_hours_to_osm_does_not_bridge_a_closed_day() -> None:
    """A closed day between two identical days must not collapse into a range covering it."""
    hours = {
        "mon": {"start": "09:00", "end": "12:00"},
        "wed": {"start": "09:00", "end": "12:00"},
    }
    assert open_hours_to_osm(hours) == "Mo 09:00-12:00; We 09:00-12:00"


def test_open_hours_to_osm_empty_is_empty_string() -> None:
    """A canteen with no open days converts to an empty string (the scraper then skips it)."""
    assert open_hours_to_osm({}) == ""


def test_studierendenwerk_schema_accepts_minimal_valid_row() -> None:
    """A row matching every rule must validate cleanly (positive control)."""
    StudierendenwerkSchema.validate(_row_with())


def test_studierendenwerk_schema_rejects_duplicate_canteen() -> None:
    """`StudierendenwerkSchema` must reject a duplicated `canteen_id`."""
    duplicated = pl.DataFrame(
        {
            "canteen_id": ["mensa-garching", "mensa-garching"],
            "name": ["Mensa Garching", "Mensa Garching"],
            "opening_hours": ["Mo-Fr 10:45-14:15", "Mo-Fr 10:45-14:15"],
            "last_update": ["2026-06-05", "2026-06-05"],
            "source_url": ["https://x.tld", "https://x.tld"],
        },
        schema=StudierendenwerkSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        StudierendenwerkSchema.validate(duplicated)


def test_studierendenwerk_schema_rejects_empty_opening_hours() -> None:
    """An empty `opening_hours` string must be rejected."""
    with pytest.raises(dy.exc.ValidationError):
        StudierendenwerkSchema.validate(_row_with(opening_hours=""))


@pytest.mark.parametrize("osm", ["lecture: Mo-Fr 08:00-20:00", "Mo-Fr 08:00-20:00; break: 12:00-13:00"])
def test_studierendenwerk_schema_rejects_macros(osm: str) -> None:
    """`lecture:`/`break:` macros must be rejected; only plain OSM is supported."""
    with pytest.raises(dy.exc.ValidationError):
        StudierendenwerkSchema.validate(_row_with(opening_hours=osm))


@pytest.mark.parametrize("url", ["www.example.tld", "ftp://example.tld", "/relative", ""])
def test_studierendenwerk_schema_rejects_non_http_source_url(url: str) -> None:
    """`source_url` must be an absolute http(s) URL."""
    with pytest.raises(dy.exc.ValidationError):
        StudierendenwerkSchema.validate(_row_with(source_url=url))


@pytest.mark.parametrize("bad_date", ["2026/06/05", "05-06-2026", "2026-6-5", "not-a-date"])
def test_studierendenwerk_schema_rejects_non_iso_last_update(bad_date: str) -> None:
    """`last_update` must be a `YYYY-MM-DD` date."""
    with pytest.raises(dy.exc.ValidationError):
        StudierendenwerkSchema.validate(_row_with(last_update=bad_date))


def test_studierendenwerk_schema_rejects_missing_column() -> None:
    """`StudierendenwerkSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"canteen_id": ["mensa-garching"]})
    with pytest.raises(dy.exc.SchemaError):
        StudierendenwerkSchema.validate(incomplete)
