#!/bin/sh

set -e # fail on first error

echo "----"
echo "configure MeiliSearch"
echo "----"
export CURL_ERROR_ARGS=--connect-timeout 5 --max-time 10 --retry 10 --retry-delay 5 --retry-max-time 60 --retry-connrefused
export CURL_COMMON_ARGS=$CURL_ERROR_ARGS --header 'content-type: application/json' -i

echo
echo "> Set primary-key"
echo
curl $CURL_COMMON_ARGS --request POST 'http://localhost:7700/indexes' --data '{ "uid": "entries", "primaryKey": "ms_id" }'
echo
echo
echo "> Set filterable attributes"
echo
curl $CURL_COMMON_ARGS --request POST 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --data '["facet"]'

echo
echo "> Upload entries data:"
echo
ls -lah "./search_data.json"
curl $CURL_COMMON_ARGS --request PUT 'http://localhost:7700/indexes/entries/documents' --data-binary "@./search_data.json"

echo
echo
echo "> Configure index:"
echo
curl $CURL_COMMON_ARGS --request POST 'http://localhost:7700/indexes/entries/settings/ranking-rules' --data '["words", "typo", "rank:desc", "exactness", "proximity", "attribute"]'

echo "synonyms:"
ls -lah "./search_synonyms.json"
curl $CURL_COMMON_ARGS --request POST 'http://localhost:7700/indexes/entries/settings/synonyms' --data "@./search_synonyms.json"
curl $CURL_COMMON_ARGS --request POST 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --data '["ms_id", "name", "arch_name", "type", "type_common_name", "parent_building", "parent_keywords", "address", "usage"]'
