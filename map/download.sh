#!/bin/bash

set -o errexit

cd "$(dirname "$0")"

echo "-- cleanup $(dirname "$0") --"
rm     ./data/*.osm.pbf
rm -rf ./gtfs_feeds/*
mkdir --parents gtfs_feeds
mkdir --parents data/transit_tiles

echo -- download dependencys --
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

echo -- download gtfs feeds --
cd gtfs_feeds || exit 1
# aah, MVV, why do you do this like this??? So stupid..
# TODO: look into replacing with https://www.delfi.de/de/leistungen-produkte/daten-dienste/
# TODO: look into replacing with https://www.transit.land/feeds?search=germany
# TODO: look into replacing with https://github.com/transitland/transitland-atlas/pull/1268
wget --tries=5 https://www.opendata-oepnv.de/dataset/17065229-c3fd-46d7-84a9-aae55aadbf40/resource/927d0830-2a40-4702-acc6-f5716352b666/download/gtfs_mvv_mitshape_240814.zip --output-document mvv.zip --tries=5 --random-wait --wait=5
unzip -q -d mvv mvv.zip
rm mvv.zip
cd .. || exit 1
