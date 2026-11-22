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
# Stage the new graph input alongside the previous one so publication is atomic.
work_dir=$(mktemp -d "$output_dir/.prepare.XXXXXX")
trap 'rm -rf "$work_dir"' EXIT

if [ -z "$base_extract" ]; then
  base_extract="$work_dir/oberbayern.osm.pbf"
  curl --fail --location --retry 5 --silent --show-error --output "$base_extract" \
    https://download.geofabrik.de/europe/germany/bayern/oberbayern/oberbayern-latest.osm.pbf
elif [ ! -s "$base_extract" ]; then
  printf 'Missing Oberbayern extract at %s.\n' "$base_extract" >&2
  exit 1
fi

jq -er '.[] | [.name, .source, (.bbox | map(tostring) | join(","))] | @tsv' "$campuses" > "$work_dir/campuses.tsv"
set -- "$base_extract"
while IFS="$(printf '\t')" read -r name source bbox; do
  printf 'Preparing the %s routing extract.\n' "$name"
  curl --fail --location --retry 5 --silent --show-error --output "$work_dir/source.osm.pbf" "$source"
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
mv -f "$work_dir/routing.osm.pbf" "$routing_extract"
for previous_extract in "$output_dir"/routing-*.osm.pbf; do
  if [ "$previous_extract" != "$routing_extract" ]; then
    rm -f "$previous_extract"
  fi
done
