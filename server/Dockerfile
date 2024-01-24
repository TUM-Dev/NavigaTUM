FROM    rust:1.75-alpine AS compiler

# to ache the build this line inludes all the dependencys all servers need
# this is not an issue since we copy the generated binary to a more minimal envornment
# Descriptions:
# - musl-dev is needed fpr musl to compile the binary
# - libpq-dev is needed for diesel (sigh)
# - libwebp-dev is until the `image` has finsihed the refactoring effort
RUN     apk add -q --update-cache --no-cache musl-dev libpq-dev libwebp-dev

WORKDIR /compiler
ENV     USER=root
ENV     RUSTFLAGS="-C target-feature=-crt-static"

# (probably cached) first run of the image build => only dependencies
COPY    ./Cargo.* ./
COPY    main-api/Cargo.* ./main-api/
COPY    feedback/Cargo.* ./feedback/
RUN     mkdir ./main-api/src/ ./main-api/src/setup/ ./feedback/src/ \
     && echo "fn main() { println!(\"Hello, world!\");}" > ./main-api/src/setup/mod.rs \
     && echo "fn main() { println!(\"Hello, world!\");}" > ./main-api/src/main.rs\
     && echo "fn main() { println!(\"Hello, world!\");}" > ./feedback/src/main.rs

RUN     cargo build --release --workspace \
     && rm -fr target/release/deps/navigatum*

# second run of the image build (including our code)
COPY    .sqlx .sqlx

COPY    feedback/src ./feedback/src
COPY    main-api/src ./main-api/src
COPY    main-api/migrations ./main-api/migrations
RUN     cargo build --release --workspace


# RUN
FROM alpine:3.19

ARG     GIT_COMMIT_SHA
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

RUN     apk update --quiet \
     && apk add -q --no-cache libgcc libpq-dev libwebp tini curl

# add `navigatum-*` to the `/bin` so we can run it from anywhere and it's easy to find.
COPY    --from=compiler /compiler/target/release/navigatum-* /bin/

# after this point, we don't need the root user anymore
RUN    adduser -D navigatum \
     && mkdir -p /home/navigatum \
     && chown -R navigatum:navigatum /home/navigatum
WORKDIR /home/navigatum
USER    navigatum

ENTRYPOINT ["tini", "--"]

# Healthchecks and exopsing ports is ommited from the dockerfile, as this
# - is done by the kubernetes deployment
# - is not needed for local development
# - would prevent us merging the different servers into one dockerfile

# These are coomands that can be chosen to run
# - a respective server or
# - an admin task
# CMD     /bin/navigatum-feedback
CMD     /bin/navigatum-main-api
