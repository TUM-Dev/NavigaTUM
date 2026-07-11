# https://docs.docker.com/build/attestations/sbom/#scan-build-context
ARG BUILDKIT_SBOM_SCAN_CONTEXT=true

# Extract data files from the data image
FROM ghcr.io/astral-sh/uv:python3.14-bookworm AS data-stage
WORKDIR /app

ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
ENV UV_LINK_MODE=copy
ENV UV_COMPILE_BYTECODE=1
ENV UV_PROJECT_ENVIRONMENT=/usr/local

# install runtime dependencies (no dev group, no project install)
COPY pyproject.toml uv.lock .python-version ./
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --frozen --no-dev --no-install-project

# collect data
COPY data/sources/ sources/
COPY data/external/ external/
COPY data/processors/ processors/
COPY data/*.py ./
COPY data/translations.yaml translations.yaml

COPY data/output output

RUN python3 compile.py \
    && test -f "./output/status_data.json" \
    && test -f "./output/status_data.parquet" \
    && test -f "./output/search_data.parquet" \
    && test -f "./output/api_data.json" \
    && test -f "./output/alias_data.parquet"
RUN cp external/results/public_transport.parquet output/public_transport.parquet \
    && cp -r sources/img/* output \
    && cp -r output/maps/site_plans output/maps/roomfinder \
    && cp -r output/maps/overlays output/maps/overlay

FROM    rust:1.97-alpine AS compiler

# to ache the build this line inludes all the dependencies all servers need
# this is not an issue since we copy the generated binary to a more minimal envornment
# Descriptions:
# - musl-dev is needed for musl to compile the binary
# - mold is used to link faster
# - I somehow could not get openssl to cooperate => we are continuing with libpq-dev
# - clang is needed for rustls to use the aws' fips complient library
RUN     apk --update add --quiet --update-cache --no-cache musl-dev libpq-dev mold clang

WORKDIR /compiler
ENV     USER=root
ENV     RUSTFLAGS="-C target-feature=-crt-static -C link-arg=-fuse-ld=mold"
# since our builds are from-scratch, we don't need incremental compliation here
ENV     CARGO_INCREMENTAL=0
# crates.io downloads occasionally drop mid-transfer on CI runners ("unexpected
# eof while reading"), failing the whole image build. Cargo's default of 3
# retries is not enough under that flakiness, so retry harder.
ENV     CARGO_NET_RETRY=10
# HTTP/2 multiplexing pools many crate downloads onto a single connection, so a
# single libcurl framing error (CURLE_HTTP2 / "Send failure: Connection reset by
# peer") tears down every in-flight transfer and CARGO_NET_RETRY can't recover.
# Forcing HTTP/1.1 trades parallelism for one independent connection per crate.
ENV     CARGO_HTTP_MULTIPLEXING=false
# added in the build
ARG     GIT_COMMIT_SHA=development
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
# possible values "release"|"debug"
ARG     PROFILE=release
ENV     PROFILE=${PROFILE}

# (probably cached) first run of the image build => only dependencies
COPY    Cargo.* ./
COPY    server/Cargo.* ./server/
COPY    motis-openapi-progenitor/Cargo.* ./motis-openapi-progenitor/
RUN     mkdir -p ./server/src/ \
        && echo "fn main() { println!(\"Hello, world!\");}" > server/src/main.rs \
        && mkdir -p ./motis-openapi-progenitor/src/ \
        && : > motis-openapi-progenitor/src/lib.rs \
        && if [ $PROFILE == "release" ]; then cargo build --release; else cargo build; fi \
        && rm -fr target/${PROFILE}/deps/navigatum* target/${PROFILE}/deps/*motis_openapi_progenitor*

# second run of the image build (including our code)
COPY    server/.sqlx server/.sqlx

COPY    motis-openapi-progenitor/src motis-openapi-progenitor/src
COPY    motis-openapi-progenitor/README.md motis-openapi-progenitor/README.md
COPY    server/src server/src
COPY    server/migrations server/migrations
RUN     if [ $PROFILE == "release" ]; then \
  cargo build --release; \
  else \
  cargo build; \
  fi


# RUN
FROM alpine:3.24 AS production-stage

# Copy all static files from the data image (images, maps, sitemaps, data files)
COPY    --from=data-stage /app/output/ /cdn

ARG     GIT_COMMIT_SHA=development
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
# possible values "release"|"debug"
ARG     PROFILE=release
ENV     PROFILE=${PROFILE}
ENV     RUST_BACKTRACE=1

RUN     apk update --quiet \
     && apk add --quiet --no-cache libgcc tini curl libpq-dev git

# add `navigatum-*` to the `/bin` so we can run it from anywhere and it's easy to find.
COPY    --from=compiler /compiler/target/${PROFILE}/navigatum-* /bin/

# after this point, we don't need the root user anymore
RUN    adduser -D navigatum \
     && mkdir -p /home/navigatum \
     && chown -R root:root /cdn/ \
     && chmod -R 555 /cdn/ \
     && chown -R navigatum:navigatum /home/navigatum
WORKDIR /home/navigatum
USER    navigatum

ENTRYPOINT ["tini", "--"]

HEALTHCHECK CMD curl --fail localhost:3003/api/status || exit 1
CMD     ["/bin/navigatum-server"]
