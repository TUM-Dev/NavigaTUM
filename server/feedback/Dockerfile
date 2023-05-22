# Compile
FROM    rust:1.69-alpine AS compiler

RUN     apk add -q --update-cache --no-cache build-base openssl-dev

# first run of the image build (probably cached, just dependencys)
RUN USER=root cargo new --bin nav
WORKDIR /nav
COPY    feedback/Cargo.* ./
RUN     RUSTFLAGS="-C target-feature=-crt-static" cargo build --release \
     && rm ./target/release/deps/nav*
# second run of the image build (including our code)
COPY    feedback/src ./src
RUN     RUSTFLAGS="-C target-feature=-crt-static" cargo build --release

# RUN
FROM alpine:3.18

RUN     apk update --quiet \
     && apk add -q --no-cache openssl1.1-compat libgcc tini

# add `navigatum-feedback` to the `/bin` so we can run it from anywhere and it's easy to find.
COPY    --from=compiler /nav/target/release/navigatum-feedback /bin/navigatum-feedback

ARG     GIT_COMMIT_SHA
ENV     GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

EXPOSE  8070

ENTRYPOINT ["tini", "--"]
HEALTHCHECK --start-period=20m  --timeout=10s CMD curl --fail localhost:8070/api/feedback/status || exit 1
CMD     /bin/navigatum-feedback
