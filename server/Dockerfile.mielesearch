FROM getmeili/meilisearch:v0.27.2

RUN apk add --no-cache jq

COPY ./configure_MeileSearch.sh /configure_MeileSearch.sh

# Get configuration data
ADD https://nav.tum.sexy/cdn/search_data.json search_data.json
ADD https://nav.tum.sexy/cdn/search_synonyms.json search_synonyms.json

EXPOSE 7700
ENTRYPOINT /configure_MeileSearch.sh & meilisearch|grep -v ' "GET /health HTTP/1.1" 200'