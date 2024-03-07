#!/usr/bin/env sh

set -e # fail on first error

mkdir -p ../dist
rm -fr ../dist

npm run build-only
rsync -r dist/* ../dist

# compress data (only using gzip, because brotli on ngnix is a royal pain)
gzip --force --keep --recursive ../dist
