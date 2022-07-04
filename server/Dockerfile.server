FROM python:3.10.5-bullseye as build-stage
# get api_data.json
ADD https://nav.tum.sexy/cdn/api_data.json data/api_data.json
# For local testing if roomapi is not avalible:
# follow the data-docs to get api_data.json, copy it to the directory server/data and enable
# COPY data/api_data.json data/api_data.json
COPY load_api_data_to_db.py load_api_data_to_db.py
RUN python3 load_api_data_to_db.py

FROM rust:1.62.0

# Create a new empty shell project
RUN USER=root cargo new --bin navigatum-server
WORKDIR /navigatum-server

# Copy our manifests
COPY ./Cargo.* ./

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# gather final dependencys
COPY ./src ./src
COPY --from=build-stage /data/api_data.db ./data/api_data.db
ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}


# Build for release.
RUN rm ./target/release/deps/navigatum_server*
RUN cargo install --path .

EXPOSE 8080
HEALTHCHECK --start-period=20m  --timeout=10s CMD curl --fail localhost:8080/api/health || exit 1
CMD ["navigatum-server"]