#!/usr/bin/bash

# if there SIGINT, SIGTERM or EXIT are signaled:
#  - kill all subshells and
#  - stop+delete mielesearch
trap 'trap - SIGTERM && docker rm $(docker stop --time 0 $(docker ps -a -q --filter ancestor=search --format="{{.ID}}")) && kill -- -$$' SIGINT SIGTERM EXIT
set -e

echo "starting meili"
(
TMP="$(mktemp --directory)"
echo "using $TMP as temporary directory for meili"
docker run -v "$TMP/meili_data":/meili_data -t msinit || exit
docker run -v "$TMP/meili_data":/meili_data -p 7700:7700 getmeili/meilisearch:latest || exit
) &
sleep 1 # to make sure, that the meili-log is before any other log :)

echo "starting the server"
(
  cd server/main-api || exit
  cargo run
  exit
) &

echo "starting the webclient"
cd webclient || exit
gulp
python -m http.server
