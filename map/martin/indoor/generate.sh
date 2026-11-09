#!/usr/bin/env bash
# Renders one `indoor_level_<L>.mbtiles` per floor with `martin cp --mode bulk`.
#
# Expects an osm2pgsql-loaded database (see map/osm2pgsql/style.lua) reachable via $DATABASE_URL
# and `martin` (a build with bulk mode, >= 2.0.0-beta.6) plus `psql` and `sqlite3` on the PATH.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="${OUT_DIR:-$HERE/out}"
# Keep in sync with INDOOR_TILESET_LEVELS in webclient/app/composables/FloorControl.ts.
LEVELS="${LEVELS:--2 -1 0 1 2 3 4 5 6 7}"
MIN_ZOOM="${MIN_ZOOM:-16}"
MAX_ZOOM="${MAX_ZOOM:-18}"
: "${DATABASE_URL:?DATABASE_URL must point at the osm2pgsql database}"

mkdir -p "$OUT_DIR"
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -v "levels={${LEVELS// /,}}" -f "$HERE/prepare.sql"

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

for level in $LEVELS; do
  echo "-> level $level"
  config="$workdir/level_${level}.yaml"
  sed "s/@LEVEL@/${level}/g" "$HERE/config.template.yaml" >"$config"
  out="$OUT_DIR/indoor_level_${level}.mbtiles"
  rm -f "$out"
  martin cp --mode bulk --config "$config" --format mvt \
    --min-zoom "$MIN_ZOOM" --max-zoom "$MAX_ZOOM" \
    --set-meta "attribution=© OpenStreetMap contributors" --output-file "$out"
  # A schema-only archive is nonempty on disk, so count the tiles.
  tiles="$(sqlite3 "$out" 'SELECT count(*) FROM tiles')"
  [ "$tiles" -gt 0 ] || { echo "ERROR: no tiles written for level $level" >&2; exit 1; }
done
