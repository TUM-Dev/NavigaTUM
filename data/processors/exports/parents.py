from pathlib import Path

import polars as pl
from external.schemas.parents import ParentsSchema

OUTPUT_DIR = Path(__file__).parent.parent.parent / "output"

# `root` is filtered out of the flat dataframe, so its name must be supplied
# explicitly; matches the German half of `_("Standorte", "Sites")` injected
# by `export.extract_exported_item`.
_ROOT_NAME_DE = "Standorte"


def export_parents_parquet(df: pl.DataFrame) -> None:
    """Write `parents.parquet` - one row per (entry, ancestor) pair."""
    OUTPUT_DIR.mkdir(exist_ok=True)
    name_by_id: dict[str, str | None] = dict(zip(df["id"].to_list(), df["name"].to_list(), strict=True))
    name_by_id["root"] = _ROOT_NAME_DE

    rows: list[dict[str, object]] = []
    for row in df.iter_rows(named=True):
        rows.extend(
            {
                "key": row["id"],
                "id": parent_id,
                "name": name_by_id.get(parent_id),
            }
            for parent_id in row.get("parents") or []
        )

    out = pl.DataFrame(rows, schema=ParentsSchema.to_polars_schema())
    ParentsSchema.write_parquet(out, OUTPUT_DIR / "parents.parquet")
