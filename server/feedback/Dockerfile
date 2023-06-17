# Compile
FROM    rust:1.70-alpine AS compiler

# to ache the build this line inludes all the dependencys all servers need
# this is not an issue since we copy the generated binary to a more minimal envornment
RUN     apk add -q --update-cache --no-cache build-base openssl-dev libpq-dev libwebp-dev

WORKDIR /compiler
ENV     USER=root
ENV     RUSTFLAGS="-C target-feature=-crt-static"

# first run of the image build (probably cached => workspace wide dependencies
COPY    ./Cargo.* ./
COPY    main-api/Cargo.* ./main-api/
COPY    feedback/Cargo.* ./feedback/
COPY    calendar/Cargo.* ./calendar/
RUN     mkdir ./main-api/src/ ./feedback/src/ ./calendar/src/ \
     && echo "fn main() { println!(\"Hello, world!\");}" > ./main-api/src/main.rs\
     && echo "fn main() { println!(\"Hello, world!\");}" > ./feedback/src/main.rs\
     && echo "fn main() { println!(\"Hello, world!\");}" > ./calendar/src/main.rs\
     && echo "fn main() { println!(\"Hello, world!\");}" > ./calendar/src/scraper.rs

RUN     cargo build --release --workspace \
     && rm -fr target/release/deps/navigatum*

        # second run of the image build (including our code)
COPY    feedback/src ./feedback/src
RUN     cargo build --release --bin navigatum-feedback


# RUN
FROM alpine:3.18

RUN     apk update --quiet \
     && apk add -q --no-cache libgcc tini libwebp curl

# add `navigatum-*` to the `/bin` so we can run it from anywhere and it's easy to find.
COPY    --from=compiler /compiler/target/release/navigatum-* /bin/

ARG     GIT_COMMIT_SHA
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

ENTRYPOINT ["tini", "--"]
EXPOSE  8070
HEALTHCHECK CMD curl --fail localhost:8070/api/feedback/status || exit 1
CMD     /bin/navigatum-feedback
