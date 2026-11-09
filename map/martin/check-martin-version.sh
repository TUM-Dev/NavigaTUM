#!/usr/bin/env bash
# The indoor tiles are rendered with the same martin binary in the Dockerfile and in render-e2e.sh.
# Bumping one without the other makes the e2e gate test a different renderer than production.
set -euo pipefail
cd "$(dirname "$0")"

dockerfile_version="$(sed -n 's/^ARG MARTIN_VERSION=\(.*\)$/\1/p' Dockerfile)"
e2e_version="$(sed -n 's/^MARTIN_VERSION="\${MARTIN_VERSION:-\([^}]*\)}".*$/\1/p' test/render-e2e.sh)"

if [ -z "$dockerfile_version" ] || [ -z "$e2e_version" ]; then
  echo "ERROR: could not find MARTIN_VERSION in map/martin/Dockerfile or map/martin/test/render-e2e.sh" >&2
  exit 1
fi
if [ "$dockerfile_version" != "$e2e_version" ]; then
  echo "ERROR: MARTIN_VERSION differs: Dockerfile=$dockerfile_version, render-e2e.sh=$e2e_version" >&2
  exit 1
fi
