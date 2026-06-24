"""
Drop coordinate submissions that are already covered by our OSM dataset.

`sources/coordinates.csv` is a manual fallback: a hand-collected coordinate per
room that predates us deriving room coordinates from OpenStreetMap geometry
(rooms tagged `ref:tum`, merged in #3345). Once a room's coordinate comes from
OSM, the manual entry is redundant and should be removed so the two sources do
not silently drift apart.

For every submission this queries `https://nav.tum.de/api/locations/{id}` and
inspects `coords.source`. The merged server represents an OSM-derived coordinate
as `source == "osm"` (the `accuracy` field only ever carries `building`), so a
submission is discarded exactly when its live coordinate is OSM-sourced.

Submissions are kept when they are NOT OSM-covered, i.e. the API reports a
non-OSM source, the location is unknown to NavigaTUM (404), or the lookup fails.
Keeping on failure is deliberate: we only ever delete a row once we have
positively confirmed OSM covers it.

Usage:
    uv run python sources/filter_osm_covered_coordinates.py            # rewrite in place
    uv run python sources/filter_osm_covered_coordinates.py --dry-run  # report only
"""

from __future__ import annotations

import argparse
import asyncio
import sys
from pathlib import Path
from urllib.parse import quote

import httpx

API_URL = "https://nav.tum.de/api/locations/{id}"
CONCURRENCY = 16
RETRIES = 3
TIMEOUT = httpx.Timeout(30.0)

CSV_PATH = Path(__file__).resolve().parent / "coordinates.csv"


class Verdict:
    """How a single submission resolved against the live API."""

    OSM_COVERED = "osm_covered"  # discard: coordinate now comes from OSM.
    KEEP_OTHER_SOURCE = "keep_other_source"  # keep: still a non-OSM source.
    KEEP_NOT_FOUND = "keep_not_found"  # keep: room unknown to NavigaTUM.
    KEEP_ERROR = "keep_error"  # keep: lookup failed, do not delete on doubt.


async def classify(client: httpx.AsyncClient, sem: asyncio.Semaphore, room_id: str) -> tuple[str, str]:
    """Return (verdict, detail) for one room id."""
    url = API_URL.format(id=quote(room_id, safe=""))
    last_error = "unknown error"
    for attempt in range(RETRIES):
        async with sem:
            try:
                response = await client.get(url)
            except httpx.HTTPError as error:
                last_error = f"{type(error).__name__}: {error}"
                await asyncio.sleep(2**attempt)
                continue
        if response.status_code == 404:
            return Verdict.KEEP_NOT_FOUND, "404 not found"
        if response.status_code != 200:
            last_error = f"HTTP {response.status_code}"
            await asyncio.sleep(2**attempt)
            continue
        source = ((response.json() or {}).get("coords") or {}).get("source")
        if source == "osm":
            return Verdict.OSM_COVERED, "source=osm"
        return Verdict.KEEP_OTHER_SOURCE, f"source={source}"
    return Verdict.KEEP_ERROR, last_error


def parse_id(line: str) -> str | None:
    """Extract the id column from a raw CSV data line, or None for header/blank."""
    stripped = line.strip()
    if not stripped or stripped.startswith("id,"):
        return None
    return stripped.split(",", 1)[0]


async def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="report what would be removed without rewriting the file",
    )
    args = parser.parse_args()

    raw_lines = CSV_PATH.read_text(encoding="utf-8").splitlines(keepends=True)
    ids = [room_id for line in raw_lines if (room_id := parse_id(line)) is not None]

    sem = asyncio.Semaphore(CONCURRENCY)
    async with httpx.AsyncClient(timeout=TIMEOUT, follow_redirects=True) as client:
        verdicts = await asyncio.gather(*(classify(client, sem, room_id) for room_id in ids))

    by_id = dict(zip(ids, verdicts, strict=True))
    discarded = [room_id for room_id, (verdict, _) in by_id.items() if verdict == Verdict.OSM_COVERED]
    errored = [(room_id, detail) for room_id, (verdict, detail) in by_id.items() if verdict == Verdict.KEEP_ERROR]

    print(f"checked:   {len(ids)}")
    print(f"discarded: {len(discarded)} (OSM-covered)")
    print(f"kept:      {len(ids) - len(discarded)}")
    if errored:
        print(f"lookup failed (kept to be safe): {len(errored)}", file=sys.stderr)
        for room_id, detail in errored:
            print(f"  ! {room_id}: {detail}", file=sys.stderr)

    if not discarded:
        print("nothing covered by OSM; file unchanged.")
        return 0

    discard_set = set(discarded)
    kept_lines = [line for line in raw_lines if (room_id := parse_id(line)) is None or room_id not in discard_set]

    if args.dry_run:
        print("\nwould remove:")
        for room_id in discarded:
            print(f"  - {room_id}")
        return 0

    CSV_PATH.write_text("".join(kept_lines), encoding="utf-8")
    print(f"\nrewrote {CSV_PATH.name}, removed {len(discarded)} OSM-covered submissions.")
    return 0


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main()))
