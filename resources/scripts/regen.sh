#!/usr/bin/bash

. venv/bin/activate

echo "regenerating the data for /data"
(
cd ./data || exit
python compile.py
)


echo "regenerating the data for /server"
(
cd ./server || exit
rm -f data/*
cp ../data/*.json data/
cp ../data/output/*.json data/
python load_api_data_to_db.py
)


echo "regenerating the data for /webclient"
(
cd ./webclient || exit
rm -fr cdn
mkdir cdn
rsync -r --exclude '*.yaml' ../data/sources/img/ cdn/
cp -r ../data/external/maps/roomfinder/* cdn/maps/roomfinder
)


echo "building the mielesearch dockerfile"
(
  cd server || exit
  docker build -t search . -f ./Dockerfile.mielesearch
)
