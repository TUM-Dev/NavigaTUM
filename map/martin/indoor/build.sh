#!/usr/bin/env bash
# Runs inside the `indoor-builder` stage of map/martin/Dockerfile: starts a throwaway PostGIS server,
# ingests the extract with osm2pgsql, and bulk-renders the per-level tilesets with generate.sh.
#
# Two steps, so Docker caches the slow ingest separately from the render:
#   build.sh ingest <extract.osm.pbf>
#   build.sh render <output dir>
set -euo pipefail

STEP="${1:?ingest or render}"
ARG="${2:?extract path (ingest) or output directory (render)}"
PGBIN="$(echo /usr/lib/postgresql/*/bin)"
# Not the base image's $PGDATA: that is a VOLUME, whose contents do not persist across build steps.
PGDATA=/var/lib/indoor-pgdata
export PGHOST=localhost PGUSER=postgres PGDATABASE=navigatum
export DATABASE_URL="postgres://postgres@localhost/navigatum"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

start_postgres() {
  # Trust auth is fine: the server only listens inside this build stage and is gone with it.
  su postgres -c "$PGBIN/pg_ctl -D $PGDATA -w -o '-c fsync=off -c synchronous_commit=off -c full_page_writes=off' start"
  trap 'su postgres -c "$PGBIN/pg_ctl -D $PGDATA -m fast stop" || true' EXIT
}

case "$STEP" in
  ingest)
    install -d -o postgres -g postgres "$PGDATA"
    su postgres -c "$PGBIN/initdb -D $PGDATA --auth=trust --encoding=UTF8" >/dev/null
    start_postgres
    psql -v ON_ERROR_STOP=1 -d postgres -c "CREATE DATABASE navigatum"
    psql -v ON_ERROR_STOP=1 -c "CREATE EXTENSION postgis"
    osm2pgsql --create --slim --drop --cache 2048 \
      --database navigatum --user postgres --host localhost \
      --output=flex --style /map/osm2pgsql/style.lua "$ARG"
    ;;
  render)
    start_postgres
    OUT_DIR="$ARG" bash "$HERE/generate.sh"
    ;;
  *)
    echo "unknown step: $STEP" >&2
    exit 1
    ;;
esac
