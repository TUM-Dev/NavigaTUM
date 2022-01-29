#!/bin/sh

ROOMFINDER_URL="https://roomapi.tum.sexy"

# fail on first error
set -e

# generate a temporary WORK_DIR
WORK_DIR=$(mktemp -d -t tmp_configure_meilesearch.XXXXXX)
echo "WORK_DIR=${WORK_DIR}"

# check if tmp dir was created
if [[ ! "$WORK_DIR" || ! -d "$WORK_DIR" ]]; then
  echo "Could not create temp dir"
  exit 1
fi

# deletes the temp directory on exit
function cleanup {      
  rm -rf "$WORK_DIR"
  echo -e "\nDeleted temp working directory $WORK_DIR"
}
trap cleanup EXIT


echo "----"
echo "configure Meilesearch"
echo "----"

echo -e "\n> Set primary-key\n"
curl --silent -i -X POST 'http://127.0.0.1:7700/indexes' --header 'content-type: application/json' --data '{ "uid": "entries", "primaryKey": "ms_id" }'

echo -e "\n\n> Set filterable attributes\n"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --header 'content-type: application/json' --data '["facet"]'


SEARCH_DATA_FILE=$WORK_DIR/search_data.json
echo -e "\n\n> Downloading/moving search data to '$SEARCH_DATA_FILE'\n"
mv ./search_data.json $WORK_DIR/search_data.json
#wget --directory-prefix $WORK_DIR $ROOMFINDER_URL/cdn/search_data.json
echo "SEARCH_DATA_FILE ($SEARCH_DATA_FILE):"
ls -lah $SEARCH_DATA_FILE
echo -e "\n> Upload entries data:\n"
echo @$WORK_DIR/search_data.json
curl --silent -i -X PUT 'http://127.0.0.1:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary @$SEARCH_DATA_FILE

echo -e "\n\n> Configure index:\n"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/ranking-rules' --header 'content-type: application/json' --data '["words","typo","rank:desc","exactness","proximity","attribute"]'
#curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/synonyms' --header 'content-type: application/json' --data @../navigatum-server/search_synonyms.json
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --header 'content-type: application/json' --data '[ "ms_id", "name", "arch_name", "type", "type_common_name", "parent_building", "parent_keywords", "address", "usage" ]'
