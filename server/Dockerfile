# https://docs.docker.com/build/attestations/sbom/#scan-build-context
ARG BUILDKIT_SBOM_SCAN_CONTEXT=true

# Extract data files from the data image
FROM    ghcr.io/tum-dev/navigatum-data:main AS data-source

FROM    rust:1.92-alpine AS compiler

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
# added in the build
ARG     GIT_COMMIT_SHA=development
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
# possible values "release"|"debug"
ARG     PROFILE=release
ENV     PROFILE=${PROFILE}

# (probably cached) first run of the image build => only dependencies
COPY    Cargo.* ./
RUN     mkdir -p ./src/ \
     && echo "fn main() { println!(\"Hello, world!\");}" > ./src/main.rs \
     && if [ $PROFILE == "release" ]; then \
          cargo build --release; \
        else \
          cargo build; \
        fi \
     && rm -fr target/${PROFILE}/deps/navigatum*

# second run of the image build (including our code)
COPY    .sqlx .sqlx

COPY    src src
COPY    migrations migrations
RUN     if [ $PROFILE == "release" ]; then \
          cargo build --release; \
        else \
          cargo build; \
        fi


# RUN
FROM alpine:3.23 AS production-stage

# Copy data files from the data image
# Includes: search_data.json, api_data.json, status_data.parquet, alias_data.parquet, public_transport.parquet
COPY    --from=data-source /cdn/*.json /app/data/output/
COPY    --from=data-source /cdn/*.parquet /app/data/output/

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
     && chown -R navigatum:navigatum /home/navigatum /app
WORKDIR /home/navigatum
USER    navigatum

ENTRYPOINT ["tini", "--"]

HEALTHCHECK CMD curl --fail localhost:3003/api/status || exit 1
CMD     ["/bin/navigatum-server"]
