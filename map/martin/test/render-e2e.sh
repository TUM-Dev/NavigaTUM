#!/usr/bin/env bash
# End-to-end render gate for the martin tile stack, run on a small real geofabrik clip:
# migrations -> osm2pgsql ingest -> planetiler generate -> boot martin -> request an indoor tile
# and render the static basemap. Both martin requests are how the indoor SRID bug surfaced in
# production (HTTP 500 on the tile, then on the render the location-preview endpoint depends on).
#
# Order matches a fresh prod bring-up: migrations run before osm2pgsql because 20240505 drops the
# legacy `rooms` metadata table, and osm2pgsql then creates the `rooms`/`doors` geometry tables
# the indoor_* functions query.
#
# Expects (from the workflow): PostGIS at $PGHOST:$PGPORT (db/user/password set), docker, cwd at
# the repository root.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

# -- tunables ----------------------------------------------------------------------------------
# Garching campus: densely indoor-mapped, so the clip has tiles with walls but no doors.
BBOX="${BBOX:-11.655,48.255,11.685,48.272}" # left,bottom,right,top
# MI building, above indoor_walls' minzoom (16.5).
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
# osm2pgsql needs PostGIS before it creates geometry columns.
psql -v ON_ERROR_STOP=1 -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" \
  -c "CREATE EXTENSION IF NOT EXISTS postgis;"
# sqlx applies migrations in version-prefix order, which is filename order.
for f in $(ls server/migrations/*.sql | sort); do
  echo "-> $f"
  psql -v ON_ERROR_STOP=1 -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" -f "$f"
done
endlog

# -- 2. osm2pgsql ingest of the real clip ------------------------------------------------------
log "osm2pgsql ingest"
# osm2pgsql-init flags from compose.yml (smaller --cache); --network host reaches PostGIS on localhost.
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

# Guard against an empty clip: a render over zero rooms would pass vacuously.
log "sanity-check ingested indoor data"
rooms=$(psql -tA -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE" -c "SELECT count(*) FROM rooms;")
echo "rooms ingested: $rooms"
if [ "$rooms" -eq 0 ]; then
  echo "ERROR: fixture clip produced 0 rooms; the render gate would pass vacuously." >&2
  exit 1
fi
endlog

# -- 3. planetiler generate --------------------------------------------------------------------
# The schema's only non-OSM source is the osmdata.openstreetmap.de ocean shapefile, whose slow
# size check trips planetiler's download timeout. Pre-fetch it with a patient wget into
# planetiler's default sources dir, then run without --download so it reads the local file.
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

# -- 4. boot martin against the populated DB + generated tiles ---------------------------------
# Prod mounts host map/martin over /map (compose.yml), so the gitignored fonts and maki sprites
# come from the host, not the image. Reproduce that /map and mount it into the :nightly base (the
# base the prod image is built from), which makes rebuilding the image unnecessary.
log "assemble martin runtime /map"
RUNDIR=/tmp/martin-run
rm -rf "$RUNDIR" && cp -r map/martin "$RUNDIR"
# Keep these in sync with map/martin/Dockerfile.
wget -q -O /tmp/roboto.zip https://github.com/googlefonts/roboto/releases/download/v2.138/roboto-android.zip
mkdir -p "$RUNDIR/fonts" && unzip -q -o /tmp/roboto.zip -d "$RUNDIR/fonts/"
wget -q -O /tmp/maki.zip https://github.com/mapbox/maki/zipball/main
rm -rf /tmp/maki && mkdir -p /tmp/maki "$RUNDIR/sprites/maki" && unzip -q /tmp/maki.zip -d /tmp/maki
mv /tmp/maki/mapbox-maki-*/icons/* "$RUNDIR/sprites/maki/"
# The style's martin sources are absolute prod URLs; the renderer fetches tiles over HTTP, so
# point them at this martin (root base path) instead of production. localhost:3001 works both
# from the runner (curl) and inside the container (the renderer self-fetches). The /cdn
# natural-earth raster is low-zoom only and unused at z18, so it's left alone.
sed -i "s#https://nav.tum.de/martin#http://localhost:${MARTIN_PORT}#g" "$RUNDIR/styles/navigatum-basemap.json"
endlog

log "run martin"
docker run -d --name martin-e2e \
  --network host \
  -e DATABASE_URL="$DATABASE_URL" \
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

# -- 5. assert martin serves the indoor tile and the static render -----------------------------
log "wait for martin health"
for i in $(seq 1 30); do
  if curl -fsS "http://localhost:${MARTIN_PORT}/health" >/dev/null 2>&1; then
    echo "martin healthy after ${i}s"; break
  fi
  [ "$i" -eq 30 ] && { echo "ERROR: martin never became healthy" >&2; exit 1; }
  sleep 1
done
endlog

# Curl up to 5 times: pass on 2xx, fail on 5xx (the failure the SRID bug produced), retry
# otherwise (transient :nightly timeout, http 000).
assert_serves() {
  local url=$1 out=${2:-/dev/null} code i
  for i in 1 2 3 4 5; do
    code=$(curl -s -o "$out" -w "%{http_code}" --max-time 30 "$url" || echo 000)
    echo "  attempt $i: http=$code"
    case "$code" in
      2*) return 0 ;;
      5*) echo "ERROR: $url returned $code" >&2; return 1 ;;
    esac
    sleep $((i * 2))
  done
  echo "ERROR: $url never succeeded" >&2; return 1
}

# Indoor tile bundle over the MI building -- the request that 500'd on door-less walls in prod.
IFS=, read -r clon clat czoom <<<"$RENDER_CAMERA"
read -r tx ty < <(awk -v lon="$clon" -v lat="$clat" -v z="$czoom" 'BEGIN {
  n = 2 ^ z; pi = atan2(0, -1); rad = lat * pi / 180
  printf "%d %d\n", int((lon + 180) / 360 * n), int((1 - log(sin(rad)/cos(rad) + 1/cos(rad)) / pi) / 2 * n)
}')
log "assert indoor tile $czoom/$tx/$ty"
assert_serves "http://localhost:${MARTIN_PORT}/indoor_rooms,indoor_pois,indoor_walls,indoor_doors/${czoom}/${tx}/${ty}?level=0.0"
endlog

# Static basemap render -- the location-preview path.
log "assert static basemap render"
assert_serves "http://localhost:${MARTIN_PORT}/style/navigatum-basemap/static/${RENDER_CAMERA}/${RENDER_SIZE}.png" /tmp/render.png
[ -s /tmp/render.png ] || { echo "ERROR: render returned an empty image" >&2; exit 1; }
endlog

echo "render-e2e passed"
