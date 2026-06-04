import json
from pathlib import Path

import polars as pl
from external.schemas.urls import UrlsSchema

OUTPUT_DIR = Path(__file__).parent.parent.parent / "output"


def export_urls_de_parquet(df: pl.DataFrame) -> None:
    _write_urls_parquet(df, language="de", filename="urls_de.parquet")


def export_urls_en_parquet(df: pl.DataFrame) -> None:
    _write_urls_parquet(df, language="en", filename="urls_en.parquet")


def _write_urls_parquet(df: pl.DataFrame, *, language: str, filename: str) -> None:
    OUTPUT_DIR.mkdir(exist_ok=True)
    rows: list[dict[str, object]] = []
    for row in df.iter_rows(named=True):
        links_json = row.get("props_links_json")
        if not links_json:
            continue
        for link in json.loads(links_json):
            url_val = link.get("url")
            text_val = link.get("text")
            url = url_val.get(language) if isinstance(url_val, dict) else None
            text = text_val.get(language) if isinstance(text_val, dict) else None
            rows.append({"key": row["id"], "url": url, "text": text})

    out = pl.DataFrame(rows, schema=UrlsSchema.to_polars_schema())
    UrlsSchema.write_parquet(out, OUTPUT_DIR / filename)
