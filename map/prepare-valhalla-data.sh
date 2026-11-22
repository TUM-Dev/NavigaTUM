#!/bin/sh

set -eu

output_dir=$1
campuses=$2
base_extract=${3:-}

if ! command -v osmium >/dev/null 2>&1; then
  apt-get update -qq
  DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends ca-certificates curl jq osmium-tool >/dev/null
fi

mkdir -p "$output_dir"
# Clean up any leftover staging directories from an interrupted run.
rm -rf "$output_dir"/.prepare.*
# Stage the new graph input alongside the previous one so publication is atomic.
work_dir=$(mktemp -d "$output_dir/.prepare.XXXXXX")
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if [ -z "$base_extract" ]; then
  base_extract="$work_dir/oberbayern.osm.pbf"
  curl --fail --location --retry 5 --silent --show-error --output "$base_extract" \
    https://download.geofabrik.de/europe/germany/bayern/oberbayern/oberbayern-latest.osm.pbf
elif [ ! -s "$base_extract" ]; then
  printf 'Missing Oberbayern extract at %s.\n' "$base_extract" >&2
  exit 1
fi

# Pin campus extracts to the same Geofabrik daily snapshot as Oberbayern so shared relations
# do not diverge across Geofabrik's rolling export window or stale base extracts.
base_timestamp=$(osmium fileinfo --get header.option.osmosis_replication_timestamp "$base_extract" 2>/dev/null || true)
snapshot_tag=$(printf '%s' "$base_timestamp" | sed -n 's/^[0-9][0-9]\([0-9][0-9]\)-\([0-9][0-9]\)-\([0-9][0-9]\)T.*/\1\2\3/p')

jq -er '.[] | [.name, .source, (.bbox | map(tostring) | join(","))] | @tsv' "$campuses" > "$work_dir/campuses.tsv"
set -- "$base_extract"
while IFS="$(printf '\t')" read -r name source bbox; do
  printf 'Preparing the %s routing extract.\n' "$name"
  dated_prefix=${source%-latest.osm.pbf}
  if [ -n "$snapshot_tag" ] && [ "$dated_prefix" != "$source" ]; then
    if ! curl --fail --location --retry 5 --silent --show-error --output "$work_dir/source.osm.pbf" \
      "${dated_prefix}-${snapshot_tag}.osm.pbf"; then
      printf 'Snapshot %s unavailable for %s, falling back to latest.\n' "$snapshot_tag" "$name" >&2
      curl --fail --location --retry 5 --silent --show-error --output "$work_dir/source.osm.pbf" "$source"
    fi
  else
    curl --fail --location --retry 5 --silent --show-error --output "$work_dir/source.osm.pbf" "$source"
  fi
  osmium extract --strategy smart --option types=multipolygon,restriction --bbox "$bbox" \
    --output "$work_dir/$name.osm.pbf" "$work_dir/source.osm.pbf"
  set -- "$@" "$work_dir/$name.osm.pbf"
done < "$work_dir/campuses.tsv"

osmium merge --fsync --output "$work_dir/routing.osm.pbf" "$@"
# Valhalla needs one version per OSM object, even if upstream extracts overlap in their relations.
multiple_versions=$(osmium fileinfo --extended --get data.multiple_versions "$work_dir/routing.osm.pbf")
if [ "$multiple_versions" != no ]; then
  echo 'The merged routing extract contains conflicting OSM object versions.' >&2
  exit 1
fi

# The Valhalla image detects new graph inputs by filename.
checksum=$(sha256sum "$work_dir/routing.osm.pbf")
routing_extract="$output_dir/routing-${checksum%% *}.osm.pbf"
if [ ! -f "$routing_extract" ]; then
  mv -f "$work_dir/routing.osm.pbf" "$routing_extract"
  # valhalla-scripted only rebuilds valhalla_tiles.tar / admins.sqlite when absent (and skips
  # rebuilding tiles altogether unless tile_dir is empty or use_tiles_ignore_pbf=False).
  rm -rf "$output_dir/valhalla_tiles" "$output_dir/valhalla_tiles.tar" "$output_dir/admins.sqlite"
fi
for previous_extract in "$output_dir"/routing-*.osm.pbf; do
  if [ "$previous_extract" != "$routing_extract" ]; then
    rm -f "$previous_extract"
  fi
done
