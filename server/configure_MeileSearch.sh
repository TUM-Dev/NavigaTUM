#!/bin/sh

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
  echo
  echo "Deleted temp working directory $WORK_DIR"
}
trap cleanup EXIT


echo "----"
echo "configure MeiliSearch"
echo "----"

echo
echo "> Set primary-key"
echo
curl --silent -i -X POST 'http://127.0.0.1:7700/indexes' --header 'content-type: application/json' --data '{ "uid": "entries", "primaryKey": "ms_id" }'
echo
echo
echo "> Set filterable attributes"
echo
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --header 'content-type: application/json' --data '["facet"]'


SEARCH_DATA_FILE=$WORK_DIR/search_data.json
echo
echo
echo "> Downloading/moving search data to '$SEARCH_DATA_FILE'"
echo
mv ./search_data.json "$SEARCH_DATA_FILE"
#wget --directory-prefix $WORK_DIR $ROOMFINDER_URL/cdn/search_data.json
echo "SEARCH_DATA_FILE ($SEARCH_DATA_FILE):"
ls -lah "$SEARCH_DATA_FILE"

echo
echo "> Upload entries data:"
echo
echo "@$SEARCH_DATA_FILE"
curl --silent -i -X PUT 'http://127.0.0.1:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary "@$SEARCH_DATA_FILE"

echo
echo
echo "> Configure index:"
echo
SEARCH_SYNONYMS_FILE=$WORK_DIR/search_synonyms.json
echo "SEARCH_DATA_FILE ($SEARCH_SYNONYMS_FILE):"
ls -lah "$SEARCH_SYNONYMS_FILE"
mv ./search_data.json "$SEARCH_SYNONYMS_FILE"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/ranking-rules' --header 'content-type: application/json' --data '["words","typo","rank:desc","exactness","proximity","attribute"]'
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/synonyms' --header 'content-type: application/json' --data "@$SEARCH_SYNONYMS_FILE"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --header 'content-type: application/json' --data '[ "ms_id", "name", "arch_name", "type", "type_common_name", "parent_building", "parent_keywords", "address", "usage" ]'
