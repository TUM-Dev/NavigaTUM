import dataframely as dy
import opening_hours
import polars as pl
import pytest

from external.loaders.opening_hours import load_opening_hours
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.opening_hours import OpeningHoursSchema


def _valid_row() -> dict[str, list[object]]:
    """Build a single valid opening-hours row."""
    return {
        "id": ["5603"],
        "opening_hours": ["Mo-Fr 09:00-21:00; Sa 09:00-13:00"],
        "source_url": ["https://www.ub.tum.de/en/opening-hours"],
        "last_update": ["2026-06-01"],
        "valid_from": [None],
        "valid_until": [None],
        "service": [None],
    }


def _row_with(**overrides: object) -> pl.DataFrame:
    """Build a one-row frame from the valid baseline, overriding named columns."""
    row = _valid_row()
    for key, value in overrides.items():
        row[key] = [value]
    return pl.DataFrame(row, schema=OpeningHoursSchema.to_polars_schema())


def test_committed_opening_hours_csv_satisfies_schema() -> None:
    """The committed `opening_hours.csv` must satisfy `OpeningHoursSchema` (drift gate)."""
    assert_satisfies_schema(OpeningHoursSchema, load_opening_hours())


@pytest.mark.parametrize(
    "osm",
    [
        "Mo-Fr 08:00-20:00",  # single range
        "Mo-Fr 08:00-12:00,13:00-20:00",  # multi range
        "Mo-Fr 09:00-21:00; Sa 09:00-13:00; PH off",  # multi rule
    ],
)
def test_plain_osm_fixtures_parse(osm: str) -> None:
    """Well-formed plain-OSM strings must parse."""
    assert opening_hours.validate(osm)


@pytest.mark.parametrize("osm", ["definitely not hours!!!", "Xy 99:99", ""])
def test_malformed_osm_fixtures_rejected(osm: str) -> None:
    """Malformed strings must not parse."""
    assert not opening_hours.validate(osm)


def test_opening_hours_schema_accepts_minimal_valid_row() -> None:
    """A row matching every rule must validate cleanly (positive control)."""
    OpeningHoursSchema.validate(_row_with())


def test_opening_hours_schema_rejects_empty_osm() -> None:
    """An empty `opening_hours` string must be rejected (malformed fixture)."""
    with pytest.raises(dy.exc.ValidationError):
        OpeningHoursSchema.validate(_row_with(opening_hours=""))


@pytest.mark.parametrize("osm", ["lecture: Mo-Fr 08:00-20:00", "Mo-Fr 08:00-20:00; break: 12:00-13:00"])
def test_opening_hours_schema_accepts_macros(osm: str) -> None:
    """`lecture:`/`break:` macros are valid on disk; the compile step expands them (positive control)."""
    OpeningHoursSchema.validate(_row_with(opening_hours=osm))


@pytest.mark.parametrize("url", ["www.ub.tum.de", "ftp://ub.tum.de", "/relative/path", ""])
def test_opening_hours_schema_rejects_non_http_source_url(url: str) -> None:
    """`source_url` must be an absolute http(s) URL."""
    with pytest.raises(dy.exc.ValidationError):
        OpeningHoursSchema.validate(_row_with(source_url=url))


@pytest.mark.parametrize("field", ["last_update", "valid_from", "valid_until"])
@pytest.mark.parametrize("bad_date", ["2026/06/01", "01-06-2026", "2026-6-1", "not-a-date"])
def test_opening_hours_schema_rejects_non_iso_dates(field: str, bad_date: str) -> None:
    """Date fields must be `YYYY-MM-DD`."""
    with pytest.raises(dy.exc.ValidationError):
        OpeningHoursSchema.validate(_row_with(**{field: bad_date}))


def test_opening_hours_schema_rejects_inverted_validity_range() -> None:
    """`valid_until` before `valid_from` must be rejected."""
    with pytest.raises(dy.exc.ValidationError):
        OpeningHoursSchema.validate(_row_with(valid_from="2026-09-01", valid_until="2026-04-01"))


def test_opening_hours_schema_accepts_bounded_validity_window() -> None:
    """A bounded `valid_from`/`valid_until` window must validate (positive control)."""
    OpeningHoursSchema.validate(_row_with(valid_from="2026-04-01", valid_until="2026-09-30"))
