#!/usr/bin/bash

# if there SIGINT, SIGTERM or EXIT are signaled:
#  - kill all subshells and
#  - stop+delete mielesearch
trap 'trap - SIGTERM && docker rm $(docker stop --time 0 $(docker ps -a -q --filter ancestor=search --format="{{.ID}}")) && kill -- -$$' SIGINT SIGTERM EXIT
set -e

echo "starting meili"
(docker run -p 7700:7700 -t search || exit) &
sleep 1 # to make sure, that the meili-log is before any other log :)

echo "starting the server"
(
  cd server || exit
  cargo run
  exit
) &

echo "starting the webclient"
cd webclient || exit
gulp
python -m http.server
