#!/bin/sh

ROOMFINDER_URL="https://new.roomfinder.tum.sexy"

# fail on first error
set -e

# generate a temporary WORK_DIR inside the current working directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
WORK_DIR=`mktemp -d -p "$DIR"`

# check if tmp dir was created
if [[ ! "$WORK_DIR" || ! -d "$WORK_DIR" ]]; then
  echo "Could not create temp dir"
  exit 1
fi

# deletes the temp directory on exit
function cleanup {      
  rm -rf "$WORK_DIR"
  echo "Deleted temp working directory $WORK_DIR"
}
trap cleanup EXIT


echo "----"
echo "configure Meilesearch"
echo "----"

curl --silent -i -X POST 'http://127.0.0.1:7700/indexes' --header 'content-type: application/json' --data '{ "uid": "entries", "primaryKey": "ms_id" }'

echo "\nSet filterable attributes"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --data '["facet"]'


# get the search_data from the cdn
echo "\nDownloading search data to '$WORK_DIR\search_data.json'"
wget --directory-prefix $WORK_DIR $ROOMFINDER_URL/cdn/search_data.json
echo "\nUpload entries data"
curl --silent -i -X PUT 'http://127.0.0.1:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary @$WORK_DIR\search_data.json

echo "\nConfigure index"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/ranking-rules' --data '["words","typo","rank:desc","exactness","proximity","attribute"]'
#curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/synonyms' --data @../navigatum-server/search_synonyms.json
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --data '[ "ms_id", "name", "arch_name", "type", "type_common_name", "parent_building", "parent_keywords", "address", "usage" ]'
