# Compile
FROM    rust:1.65-alpine AS compiler

RUN     apk add -q --update-cache --no-cache build-base openssl-dev

WORKDIR /nav
COPY    ./Cargo.* ./
COPY    ./src ./src
RUN     RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

# RUN
FROM alpine:3.17

RUN     apk update --quiet \
        && apk add -q --no-cache openssl1.1-compat libgcc tini

# add `navigatum-feedback` to the `/bin` so we can run it from anywhere and it's easy to find.
COPY    --from=compiler /nav/target/release/navigatum-feedback /bin/navigatum-feedback

ARG     GIT_COMMIT_SHA
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

EXPOSE  6000

ENTRYPOINT ["tini", "--"]
HEALTHCHECK --start-period=20m  --timeout=10s CMD curl --fail localhost:6000/api/feedback/health || exit 1
CMD     /bin/navigatum-feedback
