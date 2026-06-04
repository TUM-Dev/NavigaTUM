from __future__ import annotations

import logging
import math
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import backoff
import polars as pl
import requests
from utils import setup_logging

from external.schemas.public_transport import TRANSPORT_MODES, StationsSchema
from external.scraping_utils import CACHE_PATH

_logger = logging.getLogger(__name__)

STOPS_ENDPOINT = "https://api.transitous.org/api/v1/map/stops"
_COORDINATES_CSV = Path(__file__).parent.parent.parent / "sources" / "coordinates.csv"

_CLUSTER_EPS_KM = 5.0
_BBOX_PAD_KM = 1.5

# Courtesy cadence agreed with the motis maintainers; transitous is a free community service.
_INTER_REQUEST_DELAY_S = 1.5

# Equirectangular projection at mid-Bavaria — accurate enough for sub-kilometre clustering
# decisions at TUM latitudes (~47-49 deg N).
_REF_LAT_DEG = 48.5
_KM_PER_DEG_LAT = 111.0
_KM_PER_DEG_LON_AT_REF = _KM_PER_DEG_LAT * math.cos(math.radians(_REF_LAT_DEG))

_KNOWN_MODES: frozenset[str] = frozenset(TRANSPORT_MODES)


@dataclass(frozen=True)
class Bbox:
    min_lat: float
    min_lon: float
    max_lat: float
    max_lon: float

    def as_query(self) -> dict[str, str]:
        """Render as the `min=lat,lon&max=lat,lon` pair that motis expects."""
        return {
            "min": f"{self.min_lat},{self.min_lon}",
            "max": f"{self.max_lat},{self.max_lon}",
        }


_EMPTY_STATIONS_SCHEMA = {
    "id": pl.Utf8,
    "name": pl.Utf8,
    "modes": pl.List(pl.Utf8),
    "lat": pl.Float64,
    "lon": pl.Float64,
}


def _read_tum_coords() -> pl.DataFrame:
    df = pl.read_csv(_COORDINATES_CSV, schema_overrides={"id": pl.Utf8})
    return df.select(
        pl.col("lat").cast(pl.Float64),
        pl.col("lon").cast(pl.Float64),
    ).filter(pl.col("lat").is_not_null() & pl.col("lon").is_not_null())


def _union_find_clusters(edges: list[tuple[int, int]], n: int) -> list[int]:
    parent = list(range(n))

    def find(i: int) -> int:
        while parent[i] != i:
            parent[i] = parent[parent[i]]
            i = parent[i]
        return i

    for a, b in edges:
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb
    return [find(i) for i in range(n)]


def cluster_bboxes(coords: pl.DataFrame) -> list[Bbox]:
    """Single-linkage cluster `(lat, lon)` rows; emit one `_BBOX_PAD_KM`-padded bbox per cluster."""
    n = coords.height
    if n == 0:
        return []
    indexed = coords.with_row_index("idx")
    pairs = indexed.join(indexed, how="cross", suffix="_b").filter(pl.col("idx") < pl.col("idx_b"))
    pairs = pairs.with_columns(
        ((pl.col("lat") - pl.col("lat_b")) * _KM_PER_DEG_LAT).alias("dlat_km"),
        ((pl.col("lon") - pl.col("lon_b")) * _KM_PER_DEG_LON_AT_REF).alias("dlon_km"),
    )
    pairs = pairs.with_columns((pl.col("dlat_km").pow(2) + pl.col("dlon_km").pow(2)).sqrt().alias("dist_km"))
    edges = (
        pairs.filter(pl.col("dist_km") < _CLUSTER_EPS_KM)
        .select(pl.col("idx").cast(pl.Int64), pl.col("idx_b").cast(pl.Int64))
        .to_numpy()
        .tolist()
    )

    roots = _union_find_clusters([(int(a), int(b)) for a, b in edges], n)

    pad_lat = _BBOX_PAD_KM / _KM_PER_DEG_LAT
    pad_lon = _BBOX_PAD_KM / _KM_PER_DEG_LON_AT_REF
    grouped = (
        indexed.with_columns(pl.Series("root", roots))
        .group_by("root")
        .agg(
            pl.col("lat").min().alias("min_lat"),
            pl.col("lat").max().alias("max_lat"),
            pl.col("lon").min().alias("min_lon"),
            pl.col("lon").max().alias("max_lon"),
        )
        .sort("min_lat")
    )
    return [
        Bbox(
            min_lat=row["min_lat"] - pad_lat,
            min_lon=row["min_lon"] - pad_lon,
            max_lat=row["max_lat"] + pad_lat,
            max_lon=row["max_lon"] + pad_lon,
        )
        for row in grouped.iter_rows(named=True)
    ]


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException, max_tries=3)
def _fetch_stops(session: requests.Session, bbox: Bbox) -> list[dict[str, Any]]:
    response = session.get(STOPS_ENDPOINT, params=bbox.as_query(), timeout=30)
    response.raise_for_status()
    payload = response.json()
    if not isinstance(payload, list):
        raise RuntimeError(f"unexpected /map/stops payload shape: {type(payload).__name__}")
    return payload


def _normalise_modes(raw: list[Any] | None) -> list[str]:
    """Lowercase + dedupe motis mode strings; unknown values fold to `other`."""
    if not raw:
        return []
    out: list[str] = []
    seen: set[str] = set()
    for mode in raw:
        if not isinstance(mode, str):
            continue
        canonical = mode.lower()
        if canonical not in _KNOWN_MODES:
            canonical = "other"
        if canonical not in seen:
            seen.add(canonical)
            out.append(canonical)
    return out


def _stops_to_rows(stops: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Flatten motis stops into row dicts; no station-level folding yet."""
    rows = []
    for stop in stops:
        stop_id = stop.get("stopId") or stop.get("id")
        name = stop.get("name")
        lat = stop.get("lat")
        lon = stop.get("lon")
        if not stop_id or not name or lat is None or lon is None:
            continue
        rows.append(
            {
                "stop_id": str(stop_id),
                "parent_id": stop.get("parentId"),
                "name": str(name),
                "lat": float(lat),
                "lon": float(lon),
                "modes": _normalise_modes(stop.get("modes")),
            }
        )
    return rows


def _fold_to_stations(rows: pl.DataFrame) -> pl.DataFrame:
    """
    Group motis platform rows into real-world stations keyed by `name`.

    Motis only emits `parentId` on some platforms (e.g. U-Bahn) and not on others
    (e.g. bus tracks at the same station), so folding by `parentId` alone leaves
    duplicates per station. `name` is already city-prefixed in German GTFS
    (`"Garching, Forschungszentrum"`), making it a more reliable station key.
    """
    if rows.is_empty():
        return pl.DataFrame(schema=_EMPTY_STATIONS_SCHEMA)
    return (
        rows.group_by("name")
        .agg(
            pl.col("lat").mean().alias("lat"),
            pl.col("lon").mean().alias("lon"),
            pl.col("modes").list.explode(keep_nulls=False, empty_as_null=False).unique().alias("modes"),
            pl.col("parent_id").drop_nulls().first().alias("_parent_id"),
            pl.col("stop_id").sort_by(pl.col("stop_id").str.len_chars()).first().alias("_fallback_id"),
        )
        .with_columns(pl.coalesce(pl.col("_parent_id"), pl.col("_fallback_id")).alias("id"))
        .select("id", "name", "modes", "lat", "lon")
    )


def scrape_stations() -> None:
    """Write `public_transport.parquet` from the live motis `/map/stops` endpoint."""
    coords = _read_tum_coords()
    bboxes = cluster_bboxes(coords)
    _logger.info(f"Discovered {len(bboxes)} geographic clusters from {coords.height} TUM coordinates")

    all_rows: list[dict[str, Any]] = []
    with requests.Session() as session:
        for i, bbox in enumerate(bboxes):
            if i > 0:
                time.sleep(_INTER_REQUEST_DELAY_S)
            _logger.info(f"[{i + 1}/{len(bboxes)}] fetching stops in {bbox}")
            stops = _fetch_stops(session, bbox)
            _logger.info(f"  → {len(stops)} platform rows")
            all_rows.extend(_stops_to_rows(stops))

    if not all_rows:
        raise RuntimeError("motis returned no stops across all bboxes — refusing to overwrite parquet")
    stations = _fold_to_stations(pl.DataFrame(all_rows, infer_schema_length=None)).sort("id")
    stations = stations.with_columns(
        pl.col("modes").list.eval(pl.element().cast(pl.Enum(TRANSPORT_MODES))),
    )

    out_path = CACHE_PATH / "public_transport.parquet"
    df = StationsSchema.cast(stations)
    StationsSchema.write_parquet(df, out_path)
    _logger.info(f"Wrote {df.height} stations to {out_path}")


if __name__ == "__main__":
    setup_logging(level=logging.INFO)
    CACHE_PATH.mkdir(exist_ok=True)
    scrape_stations()
