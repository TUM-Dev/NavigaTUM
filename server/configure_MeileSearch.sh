#!/bin/sh

set -e # fail on first error

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

echo
echo "> Upload entries data:"
echo
curl --silent -i -X PUT 'http://127.0.0.1:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary "@./search_data.json"

echo
echo
echo "> Configure index:"
echo
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/ranking-rules' --header 'content-type: application/json' --data '["words","typo","rank:desc","exactness","proximity","attribute"]'

echo "synonyms:"
ls -lah "./search_data.json"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/synonyms' --header 'content-type: application/json' --data "@./search_synonyms.json"
curl --silent -X POST 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --header 'content-type: application/json' --data '[ "ms_id", "name", "arch_name", "type", "type_common_name", "parent_building", "parent_keywords", "address", "usage" ]'
