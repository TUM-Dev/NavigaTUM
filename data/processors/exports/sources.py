import json
from pathlib import Path

import polars as pl
from external.schemas.sources import SourcesSchema

OUTPUT_DIR = Path(__file__).parent.parent.parent / "output"


def export_sources_parquet(df: pl.DataFrame) -> None:
    """Write `sources.parquet` - one row per (entry, source) pair."""
    OUTPUT_DIR.mkdir(exist_ok=True)
    rows: list[dict[str, object]] = []
    for row in df.iter_rows(named=True):
        base_json = row.get("sources_base_json")
        if not base_json:
            continue
        rows.extend(
            {
                "key": row["id"],
                "url": source.get("url"),
                "name": source.get("name"),
                "patched": row.get("sources_patched"),
            }
            for source in json.loads(base_json)
        )

    out = pl.DataFrame(rows, schema=SourcesSchema.to_polars_schema())
    SourcesSchema.write_parquet(out, OUTPUT_DIR / "sources.parquet")
