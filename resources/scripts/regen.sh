#!/usr/bin/bash

. venv/bin/activate

echo "regenerating the data for /data"
(
cd ./data || exit
python compile.py
)


echo "regenerating the data for server/main-api"
(
cd ./server/main-api || exit
mkdir -p data
rm -f data/*
cp ../../data/*.json data/
cp ../../data/output/*.json data/
python load_api_data_to_db.py
)


echo "initalising the database for server/calendar"
(
cd ./server/calendar || exit
python init_db.py
)


echo "regenerating the data for /webclient"
(
cd ./webclient || exit
rm -fr cdn
mkdir cdn
rsync -r --exclude '*.yaml' ../data/sources/img/ cdn/
cp -r ../data/external/results/maps/roomfinder/* cdn/maps/roomfinder
)


echo "building the mielesearch dockerfile"
(
  cd server/main-api || exit
  docker build -t msinit . -f ./Dockerfile.msinit
)
