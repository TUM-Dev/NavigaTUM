# https://docs.docker.com/build/attestations/sbom/#scan-build-context
ARG BUILDKIT_SBOM_SCAN_CONTEXT=true

FROM    rust:1.80-alpine AS compiler

# to ache the build this line inludes all the dependencys all servers need
# this is not an issue since we copy the generated binary to a more minimal envornment
# Descriptions:
# - musl-dev is needed for musl to compile the binary
# - mold is used to link faster
# - I somehow could not get openssl to cooperate => we are contibuing with libpq-dev
RUN     apk add -q --update-cache --no-cache musl-dev libpq-dev mold

WORKDIR /compiler
ENV     USER=root
ENV     RUSTFLAGS="-C target-feature=-crt-static -C link-arg=-fuse-ld=mold"

# (probably cached) first run of the image build => only dependencies
COPY    Cargo.* ./
RUN     mkdir -p ./src/ \
     && echo "fn main() { println!(\"Hello, world!\");}" > ./src/main.rs

RUN     cargo build --release \
     && rm -fr target/release/deps/navigatum*

# second run of the image build (including our code)
COPY    .sqlx .sqlx

COPY    src src
COPY    migrations migrations
RUN     cargo build --release


# RUN
FROM alpine:3.20 AS production-stage

ARG     GIT_COMMIT_SHA
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

RUN     apk update --quiet \
     && apk add -q --no-cache libgcc tini curl libpq-dev

# add `navigatum-*` to the `/bin` so we can run it from anywhere and it's easy to find.
COPY    --from=compiler /compiler/target/release/navigatum-* /bin/

# after this point, we don't need the root user anymore
RUN    adduser -D navigatum \
     && mkdir -p /home/navigatum \
     && chown -R navigatum:navigatum /home/navigatum
WORKDIR /home/navigatum
USER    navigatum

ENTRYPOINT ["tini", "--"]

HEALTHCHECK CMD curl --fail localhost:3003/api/status || exit 1
CMD     /bin/navigatum-server
