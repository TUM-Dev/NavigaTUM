import orjson
import polars as pl
from external.loaders.opening_hours import load_opening_hours
from external.loaders.semesters import load_semester

from processors.public_holiday_expander import bavarian_holiday_dates, contains_ph, expand_public_holidays
from processors.semester_block_expander import Semester, contains_macro, expand_semester_blocks

# Optional keys are omitted when null, not emitted as null.
_REQUIRED_KEYS = {"opening_hours": "osm", "source_url": "source_url", "last_update": "last_update"}
_OPTIONAL_KEYS = ("valid_from", "valid_until", "service")


def merge_opening_hours(
    df: pl.DataFrame,
    *,
    schedules: pl.DataFrame | None = None,
    semesters: list[Semester] | None = None,
) -> pl.DataFrame:
    """
    Attach opening-hours schedules to their entries as an `opening_hours_json` payload.

    `lecture:`/`break:` macros are expanded against `semesters`, and `PH` rules
    against the Bavarian holidays covering those semesters, into plain OSM before
    the payload is built, so downstream only sees standard OSM with no holiday
    database required. `schedules` and `semesters` are injectable for tests; both
    default to the validated CSV.
    """
    schedules = load_opening_hours() if schedules is None else schedules
    if semesters is None:
        semesters = [Semester.from_row(row) for row in load_semester().iter_rows(named=True)]

    unknown = set(schedules["id"]) - set(df["id"])
    if unknown:
        raise ValueError(f"opening-hours schedule targets unknown entry id(s): {sorted(unknown)}")

    # The semester span bounds which holidays a `PH` rule expands into; computed once for all rows.
    holiday_years = {year for semester in semesters for year in range(semester.start.year, semester.end.year + 1)}
    holiday_dates = bavarian_holiday_dates(holiday_years)

    payloads = []
    for row in schedules.iter_rows(named=True):
        osm = expand_semester_blocks(row["opening_hours"], semesters)
        if contains_macro(osm):
            raise ValueError(f"opening-hours for entry {row['id']!r} still has macros after expansion: {osm!r}")
        if contains_ph(osm) and not holiday_dates:
            raise ValueError(
                f"opening-hours for entry {row['id']!r} uses `PH` but no Bavarian holidays were available "
                f"to expand it against; is the semester list empty? {osm!r}"
            )
        osm = expand_public_holidays(osm, holiday_dates)
        if not osm.strip():
            raise ValueError(
                f"opening-hours for entry {row['id']!r} expanded to an empty schedule; "
                f"check the semester list covers its macros: {row['opening_hours']!r}"
            )
        payload = {key: row[column] for column, key in _REQUIRED_KEYS.items()}
        payload["osm"] = osm  # expanded plain OSM (no macros, no `PH`) - not the raw on-disk string.
        payload.update({key: row[key] for key in _OPTIONAL_KEYS if row[key] is not None})
        payloads.append({"id": row["id"], "opening_hours_json": orjson.dumps(payload).decode()})

    encoded = pl.DataFrame(payloads, schema={"id": pl.Utf8(), "opening_hours_json": pl.Utf8()})
    return df.join(encoded, on="id", how="left")
