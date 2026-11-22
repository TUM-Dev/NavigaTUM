#!/bin/bash

set -o errexit

cd "$(dirname "$0")"

echo "-- cleanup $(dirname "$0") --"
rm -f  ./data/*.osm.pbf

echo -- download dependencies --
if command -v apk > /dev/null 2>&1
then
  apk --update add --quiet wget
else
  echo "[WARNING] skipping apk installation of wget. Please make sure it is installed"
fi


echo -- download geodata --
cd data || exit 1
name="${1##*/}"
base="https://download.geofabrik.de/$1"
now="$(date +%s)"
wget -nv -O "${name}-latest.osm.pbf" "${base}-latest.osm.pbf" --tries=5 --random-wait --wait=5 \
  || wget -nv -O "${name}-latest.osm.pbf" "${base}-$(date -u -d "@$((now - 86400))" +%y%m%d).osm.pbf" --tries=5 --random-wait --wait=5 \
  || wget -nv -O "${name}-latest.osm.pbf" "${base}-$(date -u -d "@$((now - 172800))" +%y%m%d).osm.pbf" --tries=5 --random-wait --wait=5
cd .. || exit 1
