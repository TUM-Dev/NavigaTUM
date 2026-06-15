#!/usr/bin/env bash
# End-to-end render gate for the martin tile stack.
#
# Reproduces a fresh production install on a small, real geofabrik clip, then asserts the
# indoor tile SQL functions don't throw on that data and that martin can render the static
# basemap image the server's location-preview endpoint depends on. This is the path that broke
# in the 2026-06-15 outage: a door-less indoor tile threw a PostGIS SRID error, which cascaded
# into failed static renders and ultimately took down the whole API.
#
# Faithful ordering (matches a fresh prod bring-up):
#   server migrations  ->  osm2pgsql ingest  ->  planetiler generate  ->  martin render
# Migrations run first because 20240505 drops the legacy `rooms` metadata table; osm2pgsql
# then creates the live `rooms`/`doors` geometry tables that the indoor_* functions query.
#
# Expects (provided by the workflow):
#   - a reachable PostGIS at $PGHOST:$PGPORT, db $PGDATABASE, user $PGUSER, $PGPASSWORD set
#   - docker available
#   - run from the repository root
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

# -- tunables ----------------------------------------------------------------------------------
# Garching research campus: the MI building (and neighbours) are densely indoor-mapped, so a
# clip here contains tiles with walls but no doors -- the exact regression trigger.
BBOX="${BBOX:-11.655,48.255,11.685,48.272}" # left,bottom,right,top
# A camera centred on the MI building at a zoom above indoor_walls' minzoom (16.5).
RENDER_CAMERA="${RENDER_CAMERA:-11.6679,48.2627,18}"
RENDER_SIZE="${RENDER_SIZE:-600x400}"
FIXTURE_PBF="${FIXTURE_PBF:-/tmp/navigatum-e2e-fixture.osm.pbf}"
MBTILES_OUT="${MBTILES_OUT:-/tmp/navigatum-e2e-output.mbtiles}"
OSM2PGSQL_IMAGE="iboates/osm2pgsql:latest"   # same image as compose.yml
MARTIN_PORT=3001
DATABASE_URL="postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${PGDATABASE}"
export PGPASSWORD

log() { echo "::group::$1"; }
endlog() { echo "::endgroup::"; }

# -- 1. server migrations ----------------------------------------------------------------------
log "apply server migrations"
# PostGIS must exist before osm2pgsql creates geometry columns; migrations may also create it,
# so guard with IF NOT EXISTS.
psql -v ON_ERROR_STOP=1 -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" \
  -c "CREATE EXTENSION IF NOT EXISTS postgis;"
# sqlx orders migrations by the numeric version prefix, which is exactly filename sort order.
for f in $(ls server/migrations/*.sql | sort); do
  echo "-> $f"
  psql -v ON_ERROR_STOP=1 -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" -f "$f"
done
endlog

# -- 2. osm2pgsql ingest of the real clip ------------------------------------------------------
log "osm2pgsql ingest"
# Prod flags from compose.yml's osm2pgsql-init, minus the oversized --cache (the clip is tiny).
# --network host so the container reaches the runner-side PostGIS service at localhost.
docker run --rm \
  --network host \
  -e PGPASSWORD \
  -v "$(dirname "$FIXTURE_PBF")":/data:ro \
  -v "$REPO_ROOT/map/osm2pgsql":/map/osm2pgsql:ro \
  "$OSM2PGSQL_IMAGE" \
  osm2pgsql --create --slim --cache 512 \
    --database "$PGDATABASE" --user "$PGUSER" --host localhost --port "$PGPORT" \
    "/data/$(basename "$FIXTURE_PBF")" \
    --output=flex --style /map/osm2pgsql/style.lua
endlog

# Fail loudly if the clip turned out to contain no indoor geometry -- a green render against an
# empty DB would be a false pass that hides exactly the kind of regression this gate exists for.
log "sanity-check ingested indoor data"
rooms=$(psql -tA -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" -c "SELECT count(*) FROM rooms;")
echo "rooms ingested: $rooms"
if [ "$rooms" -eq 0 ]; then
  echo "ERROR: fixture clip produced 0 rooms; the render gate would pass vacuously." >&2
  exit 1
fi
endlog

# -- 3. assert the tile functions don't throw on the real data (fast, precise signal) ----------
# This localises a function-level regression (like the SRID bug) before the slower render step.
log "assert indoor tile functions over the clip"
psql -v ON_ERROR_STOP=1 -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" \
  -f map/martin/test/assert_tile_functions.sql
endlog

# -- 4. planetiler generate (real small-extent import) -----------------------------------------
# The schema's only non-OSM source is the ocean shapefile from osmdata.openstreetmap.de, which
# is slow enough that planetiler's built-in downloader times out on its size check. Pre-fetch it
# with a patient, resumable wget into planetiler's default sources dir (data/sources, relative
# to this repo root), then run WITHOUT --download so planetiler just reads the local file with
# no network check. The workflow caches the dir across runs.
log "fetch planetiler sources"
PLANETILER_SOURCES="${PLANETILER_SOURCES:-data/sources}"
mkdir -p "$PLANETILER_SOURCES"
ocean_zip="$PLANETILER_SOURCES/osmdata.openstreetmap.de_download_water_polygons_split_3857.zip"
if [ ! -s "$ocean_zip" ]; then
  wget --tries=5 --timeout=180 --waitretry=15 -c -q \
    https://osmdata.openstreetmap.de/download/water-polygons-split-3857.zip -O "$ocean_zip"
fi
wget -q https://github.com/onthegomap/planetiler/releases/latest/download/planetiler.jar -O /tmp/planetiler.jar
endlog

log "planetiler generate"
java -Xmx1g -jar /tmp/planetiler.jar generate-custom \
  --schema=map/planetiler/shortbread_custom.yml \
  --osm-path="$FIXTURE_PBF" \
  --output="$MBTILES_OUT" --force
test -s "$MBTILES_OUT" || { echo "ERROR: planetiler produced no mbtiles" >&2; exit 1; }
endlog

# -- 5. boot martin against the populated DB + generated tiles ---------------------------------
# Prod runs the martin image but mounts the host map/martin over /map (compose.yml), so fonts
# and maki sprites -- which are gitignored and fetched at build time -- come from the host. We
# reproduce that runtime /map: a copy of map/martin with fonts + maki fetched exactly as
# map/martin/Dockerfile does, then mounted into the :nightly renderer (same base the prod image
# is built from; the mount shadows any baked /map, so building it again would be redundant).
log "assemble martin runtime /map"
RUNDIR=/tmp/martin-run
rm -rf "$RUNDIR" && cp -r map/martin "$RUNDIR"
# Keep these in sync with map/martin/Dockerfile.
wget -q -O /tmp/roboto.zip https://github.com/googlefonts/roboto/releases/download/v2.138/roboto-android.zip
mkdir -p "$RUNDIR/fonts" && unzip -q -o /tmp/roboto.zip -d "$RUNDIR/fonts/"
wget -q -O /tmp/maki.zip https://github.com/mapbox/maki/zipball/main
rm -rf /tmp/maki && mkdir -p /tmp/maki "$RUNDIR/sprites/maki" && unzip -q /tmp/maki.zip -d /tmp/maki
mv /tmp/maki/mapbox-maki-*/icons/* "$RUNDIR/sprites/maki/"
# The style's sources are absolute prod URLs (https://nav.tum.de/martin/...). The native
# renderer fetches tiles over HTTP, so without rewriting them the render would hit production
# instead of our local data -- defeating the test. Keep the prod /martin base path and only
# swap the host for this martin (reachable at localhost:3001 both from the runner, where curl
# runs, and from inside the container, where the renderer self-fetches).
sed -i "s#https://nav.tum.de#http://localhost:${MARTIN_PORT}#g" "$RUNDIR/styles/navigatum-basemap.json"
endlog

log "run martin"
docker run -d --name martin-e2e \
  --network host \
  -e DATABASE_URL="$DATABASE_URL" \
  -e BASE_PATH=/martin/ \
  -v "$RUNDIR":/map:ro \
  -v "$MBTILES_OUT":/data/output.mbtiles:ro \
  ghcr.io/maplibre/martin:nightly --config /map/config.yaml
endlog

cleanup() {
  echo "--- martin logs ---"
  docker logs martin-e2e 2>&1 | tail -50 || true
  docker rm -f martin-e2e >/dev/null 2>&1 || true
}
trap cleanup EXIT

# -- 6. wait for health, then render -----------------------------------------------------------
log "wait for martin health"
for i in $(seq 1 30); do
  if curl -fsS "http://localhost:${MARTIN_PORT}/martin/health" >/dev/null 2>&1; then
    echo "martin healthy after ${i}s"; break
  fi
  [ "$i" -eq 30 ] && { echo "ERROR: martin never became healthy" >&2; exit 1; }
  sleep 1
done
endlog

# The static-image render is the production-critical path (location previews). The :nightly
# renderer can time out transiently, so retry a few times: a transient timeout (curl exit 28 /
# no status) is retried, but a consistent 5xx -- the actual bug class -- exhausts retries and
# fails the gate.
render_url="http://localhost:${MARTIN_PORT}/martin/style/navigatum-basemap/static/${RENDER_CAMERA}/${RENDER_SIZE}.png"
log "assert static basemap render: $render_url"
ok=0
for i in $(seq 1 5); do
  code=$(curl -s -o /tmp/render.png -w "%{http_code}" --max-time 30 "$render_url" || echo "000")
  size=$(wc -c </tmp/render.png 2>/dev/null || echo 0)
  echo "attempt $i: http=$code bytes=$size"
  if [ "$code" = "200" ] && [ "$size" -gt 0 ]; then ok=1; break; fi
  if [ "$code" != "000" ] && [ "${code:0:1}" = "5" ]; then
    echo "ERROR: render returned $code (server-side failure, not a transient timeout)" >&2
    exit 1
  fi
  sleep $((i * 2))
done
[ "$ok" -eq 1 ] || { echo "ERROR: static render never succeeded" >&2; exit 1; }
endlog

echo "render-e2e passed"
