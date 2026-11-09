# The build context is `map/` (not `map/martin/`): the indoor builder needs `osm2pgsql/style.lua`.

# -- indoor tiles --
# Throwaway builder: runs a PostGIS server, ingests the OSM extract with osm2pgsql, and bulk-renders
# one `indoor_level_<L>.mbtiles` per floor. Only the mbtiles reach the final image.
FROM postgis/postgis:18-3.6 AS indoor-builder
# Bulk mode of `martin cp` ships in the release binaries since 2.0.0-beta.6. Keep in sync with render-e2e.sh (enforced by the martin-version-in-sync pre-commit hook).
ARG MARTIN_VERSION=2.0.0-beta.6
RUN apt-get update && \
    apt-get install --no-install-recommends --yes ca-certificates wget osm2pgsql sqlite3 && \
    rm -rf /var/lib/apt/lists/*
RUN wget -q -O /tmp/martin.tar.gz "https://github.com/maplibre/martin/releases/download/martin-v${MARTIN_VERSION}/martin-x86_64-unknown-linux-musl.tar.gz" && \
    tar -xzf /tmp/martin.tar.gz -C /usr/local/bin martin && \
    rm /tmp/martin.tar.gz
# CI passes the commit SHA, so every CI build downloads a fresh extract and re-ingests it. A URL `ADD`
# would revalidate against geofabrik instead, but BuildKit fails on its 304 responses ("invalid not-modified ETag").
# Local builds reuse the cached extract; pass `--build-arg GIT_COMMIT_SHA=$(date +%s)` to refresh it.
ARG GIT_COMMIT_SHA=local
RUN echo "extract for ${GIT_COMMIT_SHA}" && \
    mkdir -p /data && \
    wget -q -O /data/extract.osm.pbf https://download.geofabrik.de/europe/germany/bayern/oberbayern-latest.osm.pbf
COPY osm2pgsql/ /map/osm2pgsql/
COPY martin/indoor/build.sh /map/martin/indoor/build.sh
RUN /map/martin/indoor/build.sh ingest /data/extract.osm.pbf

FROM indoor-builder AS indoor-renderer
COPY martin/indoor/ /map/martin/indoor/
RUN /map/martin/indoor/build.sh render /out

# -- runtime --
FROM ghcr.io/maplibre/martin:latest

# :latest is the Ubuntu rendering build and ships without unzip.
RUN apt-get update && \
    apt-get install --no-install-recommends --yes unzip && \
    rm -rf /var/lib/apt/lists/*

# -- fonts --
RUN wget -q -O /tmp/roboto-android.zip https://github.com/googlefonts/roboto/releases/download/v2.138/roboto-android.zip && \
    unzip -q /tmp/roboto-android.zip -d "$(dirname "$0")/fonts/" && \
    rm /tmp/roboto-android.zip

# -- sprites --
RUN wget -q -O /tmp/sprites_maki.zip https://github.com/mapbox/maki/zipball/main && \
    mkdir -p "$(dirname "$0")/sprites/maki/" && \
    unzip -q /tmp/sprites_maki.zip -d /tmp/sprites && \
    rm /tmp/sprites_maki.zip && \
    mv /tmp/sprites/mapbox-maki-*/icons/* "$(dirname "$0")/sprites/maki/" && \
    rm -fr /tmp/sprites/

# -- data --
# Outside /map because compose bind-mounts over it.
COPY --from=indoor-renderer /out/ /indoor/

# -- styles --
COPY ./martin/styles /map/styles/
COPY ./martin/config.yaml /map/config.yaml

CMD ["--config", "/map/config.yaml"]
