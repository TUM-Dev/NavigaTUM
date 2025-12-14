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
wget "https://download.geofabrik.de/$1-latest.osm.pbf" --tries=5 --random-wait --wait=5
cd .. || exit 1

echo -- copy valhalla config --
cp --force valhalla.json data/
