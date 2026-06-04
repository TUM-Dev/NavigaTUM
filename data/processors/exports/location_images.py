import json
from pathlib import Path

import polars as pl
from external.schemas.location_images import LocationImagesSchema

OUTPUT_DIR = Path(__file__).parent.parent.parent / "output"


def export_location_images_parquet(df: pl.DataFrame) -> None:
    OUTPUT_DIR.mkdir(exist_ok=True)
    rows: list[dict[str, object]] = []
    for row in df.iter_rows(named=True):
        imgs_json = row.get("imgs_json")
        if not imgs_json:
            continue
        for img in json.loads(imgs_json):
            author = img.get("author") or {}
            source = img.get("source") or {}
            license_ = img.get("license") or {}
            rows.append(
                {
                    "key": row["id"],
                    "name": img.get("name"),
                    "author_url": author.get("url"),
                    "author_text": author.get("text"),
                    "source_url": source.get("url"),
                    "source_text": source.get("text"),
                    "license_url": license_.get("url"),
                    "license_text": license_.get("text"),
                }
            )

    out = pl.DataFrame(rows, schema=LocationImagesSchema.to_polars_schema())
    LocationImagesSchema.write_parquet(out, OUTPUT_DIR / "location_images.parquet")
