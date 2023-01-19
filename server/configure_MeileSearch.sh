#!/bin/bash

set -e # fail on first error
meilisearch &
MEILIPID=$(pgrep meilisearch)

curl_with_args() {
  curl \
   --connect-timeout 5 --max-time 10 --retry 30 --retry-delay 1 --retry-max-time 60 --retry-connrefused --retry-all-errors \
   --silent --show-error \
   --header 'content-type: application/json' -i \
   "$@"
}

echo "----"
echo "configure MeiliSearch"
echo "----"

echo
echo "> Set primary-key"
echo
curl_with_args --request POST 'http://localhost:7700/indexes' --data '{ "uid": "entries", "primaryKey": "ms_id" }'
echo
echo
echo "> Set filterable attributes"
echo
curl_with_args --request PUT 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --data '["facet"]'

echo
echo "> Upload entries data:"
echo
ls -lah "./search_data.json"
curl_with_args --request PUT 'http://localhost:7700/indexes/entries/documents' --data-binary "@./search_data.json"

echo "synonyms:"
ls -lah "./search_synonyms.json"
curl_with_args --request PUT 'http://localhost:7700/indexes/entries/settings/synonyms' --data "@./search_synonyms.json"
curl_with_args --request PUT 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --data '["ms_id", "name", "arch_name", "type", "type_common_name", "parent_building_names", "parent_keywords", "address", "usage"]'

echo
echo
echo "> Configure index:"
echo
curl_with_args --request PUT 'http://localhost:7700/indexes/entries/settings/ranking-rules' --data '["words", "typo", "rank:desc", "exactness", "proximity", "attribute"]'

echo
echo
echo "> Waiting for indexing to complete..."
echo
while [[ "$(curl_with_args 'http://localhost:7700/indexes/entries/stats')" == *'"isIndexing":true'* ]]; do
    curl_with_args 'http://localhost:7700/indexes/entries/stats' -silent
    sleep 5
done

# kill meilisearch by sending the `Ctrl + C` interrupt signal
kill -INT "$MEILIPID"
