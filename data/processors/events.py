import json
from datetime import datetime
from pathlib import Path

import polars as pl

DATA_DIR = Path(__file__).parent.parent
SOURCES_DIR = DATA_DIR / "sources"
OUTPUT_DIR = DATA_DIR / "output"
EXTERNAL_RESULTS = DATA_DIR / "external" / "results"

EVENTS_CSV = SOURCES_DIR / "events.csv"
ACCREDITED_HSGS_CSV = SOURCES_DIR / "accredited_hsgs.csv"
ORGS_EN_CSV = EXTERNAL_RESULTS / "orgs-en_tumonline.csv"
ORGS_DE_CSV = EXTERNAL_RESULTS / "orgs-de_tumonline.csv"

EVENT_COLUMNS = [
    "event_image",
    "event_lat",
    "event_lon",
    "event_name",
    "event_datetime_start_at",
    "event_datetime_end_at",
    "event_description",
    "event_organising_org",
]


def _validate_iso8601(values: list[str | None], column: str) -> None:
    for i, v in enumerate(values):
        if v is None:
            raise ValueError(f"events.csv row {i}: {column} is required")
        try:
            datetime.fromisoformat(v)
        except ValueError as e:
            raise ValueError(f"events.csv row {i}: {column}={v!r} is not ISO 8601") from e


def export_events_parquet() -> None:
    """Read events.csv, validate, write events.parquet.

    Stored datetimes are kept as ISO 8601 strings so the Rust parquet reader can
    parse them with chrono::DateTime::parse_from_rfc3339 without depending on
    Polars datetime serialization specifics.
    """
    if not EVENTS_CSV.exists():
        # No events defined — emit empty parquet so the server loader has a file.
        df = pl.DataFrame(schema={
            "image": pl.Utf8,
            "lat": pl.Float64,
            "lon": pl.Float64,
            "name": pl.Utf8,
            "starts_at": pl.Utf8,
            "ends_at": pl.Utf8,
            "description": pl.Utf8,
            "organising_org": pl.Utf8,
        })
    else:
        raw = pl.read_csv(
            EVENTS_CSV,
            schema_overrides={
                "event_image": pl.Utf8,
                "event_lat": pl.Float64,
                "event_lon": pl.Float64,
                "event_name": pl.Utf8,
                "event_datetime_start_at": pl.Utf8,
                "event_datetime_end_at": pl.Utf8,
                "event_description": pl.Utf8,
                "event_organising_org": pl.Utf8,
            },
        )
        missing = [c for c in EVENT_COLUMNS if c not in raw.columns]
        if missing:
            raise ValueError(f"events.csv missing columns: {missing}")

        _validate_iso8601(raw["event_datetime_start_at"].to_list(), "event_datetime_start_at")
        _validate_iso8601(raw["event_datetime_end_at"].to_list(), "event_datetime_end_at")

        for i, (start, end) in enumerate(
            zip(raw["event_datetime_start_at"], raw["event_datetime_end_at"], strict=True)
        ):
            if datetime.fromisoformat(end) < datetime.fromisoformat(start):
                raise ValueError(f"events.csv row {i}: end {end} is before start {start}")

        df = raw.rename(
            {
                "event_image": "image",
                "event_lat": "lat",
                "event_lon": "lon",
                "event_name": "name",
                "event_datetime_start_at": "starts_at",
                "event_datetime_end_at": "ends_at",
                "event_description": "description",
                "event_organising_org": "organising_org",
            }
        ).select("image", "lat", "lon", "name", "starts_at", "ends_at", "description", "organising_org")

    OUTPUT_DIR.mkdir(exist_ok=True)
    df.write_parquet(OUTPUT_DIR / "events.parquet", use_pyarrow=True, compression_level=22)


def export_known_event_orgs() -> None:
    """Combine TUMonline orgs + accredited HSGs into known_event_orgs.json.

    Frontend uses this list to populate the organising-org dropdown. The events
    CSV/DB column itself remains free-form — this file is purely a UI hint.
    """
    orgs: list[dict[str, str | None]] = []

    if ORGS_EN_CSV.exists() and ORGS_DE_CSV.exists():
        en = pl.read_csv(ORGS_EN_CSV, schema_overrides={"org_id": pl.Utf8}).rename({"name": "name_en"})
        de = pl.read_csv(ORGS_DE_CSV, schema_overrides={"org_id": pl.Utf8}).rename({"name": "name_de"})
        merged = en.select("code", "name_en").join(de.select("code", "name_de"), on="code", how="left")
        for row in merged.to_dicts():
            orgs.append(
                {
                    "kind": "tumonline",
                    "code": row["code"],
                    "name_de": row["name_de"] or row["name_en"],
                    "name_en": row["name_en"],
                    "url": None,
                }
            )

    if ACCREDITED_HSGS_CSV.exists():
        hsgs = pl.read_csv(ACCREDITED_HSGS_CSV)
        for row in hsgs.to_dicts():
            orgs.append(
                {
                    "kind": "hsg",
                    "code": row["code"],
                    "name_de": row["name_de"],
                    "name_en": row["name_en"],
                    "url": row.get("url"),
                }
            )

    OUTPUT_DIR.mkdir(exist_ok=True)
    with (OUTPUT_DIR / "known_event_orgs.json").open("w", encoding="utf-8") as f:
        json.dump(orgs, f, indent=2, ensure_ascii=False)
        f.write("\n")
