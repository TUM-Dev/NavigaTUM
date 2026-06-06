import orjson
import polars as pl
from external.loaders.opening_hours import load_opening_hours

# Optional keys are omitted when null, not emitted as null.
_REQUIRED_KEYS = {"opening_hours": "osm", "source_url": "source_url", "last_update": "last_update"}
_OPTIONAL_KEYS = ("valid_from", "valid_until", "service")


def merge_opening_hours(df: pl.DataFrame, *, schedules: pl.DataFrame | None = None) -> pl.DataFrame:
    """
    Attach opening-hours schedules to their entries as an `opening_hours_json` payload.

    `schedules` is injectable for tests; it defaults to the validated CSV.
    """
    schedules = load_opening_hours() if schedules is None else schedules

    unknown = set(schedules["id"]) - set(df["id"])
    if unknown:
        raise ValueError(f"opening-hours schedule targets unknown entry id(s): {sorted(unknown)}")

    payloads = []
    for row in schedules.iter_rows(named=True):
        payload = {key: row[column] for column, key in _REQUIRED_KEYS.items()}
        payload.update({key: row[key] for key in _OPTIONAL_KEYS if row[key] is not None})
        payloads.append({"id": row["id"], "opening_hours_json": orjson.dumps(payload).decode()})

    encoded = pl.DataFrame(payloads, schema={"id": pl.Utf8(), "opening_hours_json": pl.Utf8()})
    return df.join(encoded, on="id", how="left")
